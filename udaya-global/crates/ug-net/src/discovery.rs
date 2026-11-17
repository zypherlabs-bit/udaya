//! Distributed peer discovery: Kademlia-style routing, authenticated peer
//! exchange, address validation and peer-diversity controls.
//!
//! The routing table is a bounded Kademlia XOR table. Lookups are iterative:
//! the node asks the closest known peers for the target, merges the answers,
//! and repeats until the closest known peers have all been queried. Every
//! answer is authenticated by the existing Noise/Ed25519 session, and every
//! advertised address is validated before it is stored or dialled.
//!
//! Bootstrap nodes are ordinary peers. They have no consensus authority, and
//! the node continues to operate from its persistent peer store when every
//! bootstrap address is unreachable.

use crate::identity::PeerId;
use crate::message::PeerAddress;
use crate::peer::RoutingTable;
use crate::{Error, Result};
use std::collections::{BTreeSet, HashMap};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::{Duration, Instant};

/// Default Kademlia bucket size.
pub const DEFAULT_DHT_K: usize = 20;
/// Default number of peers queried in parallel during a lookup.
pub const DEFAULT_LOOKUP_PARALLELISM: usize = 3;
/// Default lookup timeout.
pub const DEFAULT_LOOKUP_TIMEOUT: Duration = Duration::from_secs(10);
/// Maximum address entries accepted in one peer-exchange message.
pub const MAX_EXCHANGE_PEERS: usize = 64;

/// Address acceptance policy.
///
/// Production defaults reject loopback and private addresses, which prevents
/// a malicious peer from poisoning the routing table with unreachable or
/// duplicate identities on a single host. Tests and local development use
/// [`AddressPolicy::local_testing`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddressPolicy {
    pub allow_private: bool,
    pub allow_loopback: bool,
}

impl Default for AddressPolicy {
    fn default() -> Self {
        Self {
            allow_private: false,
            allow_loopback: false,
        }
    }
}

impl AddressPolicy {
    /// Permissive policy for local tests and private testnets.
    pub fn local_testing() -> Self {
        Self {
            allow_private: true,
            allow_loopback: true,
        }
    }
}

fn is_private(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()
                || v4.is_loopback()
                || v4.is_link_local()
                || v4.octets()[0] == 0
        }
        IpAddr::V6(v6) => {
            let segments = v6.segments();
            // fc00::/7 unique-local, fe80::/10 link-local, ::1 loopback.
            (segments[0] & 0xfe00) == 0xfc00
                || (segments[0] & 0xffc0) == 0xfe80
                || v6.is_loopback()
        }
    }
}

/// Validate an advertised peer address before it is stored or dialled.
pub fn validate_peer_address(
    address: SocketAddr,
    local: Option<SocketAddr>,
    policy: &AddressPolicy,
) -> Result<()> {
    if address.port() == 0 {
        return Err(Error::Peer("peer address has port zero".into()));
    }
    let ip = address.ip();
    if ip.is_unspecified() {
        return Err(Error::Peer("peer address is unspecified".into()));
    }
    if ip.is_multicast() {
        return Err(Error::Peer("peer address is multicast".into()));
    }
    if ip.is_loopback() && !policy.allow_loopback {
        return Err(Error::Peer("peer address is loopback".into()));
    }
    if is_private(&ip) && !policy.allow_private {
        return Err(Error::Peer("peer address is private".into()));
    }
    if let Some(local) = local {
        if address == local {
            return Err(Error::Peer("peer address is the local listen address".into()));
        }
    }
    Ok(())
}

/// Peer-diversity policy: cap the number of peers per network subnet.
///
/// This limits the damage a single operator (or a Sybil attacker controlling
/// one address block) can do to the routing table and outbound selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiversityPolicy {
    pub max_per_subnet: usize,
    pub ipv4_prefix: u8,
    pub ipv6_prefix: u8,
}

impl Default for DiversityPolicy {
    fn default() -> Self {
        Self {
            max_per_subnet: 8,
            ipv4_prefix: 24,
            ipv6_prefix: 64,
        }
    }
}

impl DiversityPolicy {
    fn validate(&self) -> Result<()> {
        if self.max_per_subnet == 0 {
            return Err(Error::Peer("max_per_subnet must be non-zero".into()));
        }
        if self.ipv4_prefix > 32 || self.ipv6_prefix > 128 {
            return Err(Error::Peer("invalid diversity subnet prefix".into()));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum SubnetKey {
    V4([u8; 4]),
    V6([u8; 16]),
}

fn subnet_key(address: &SocketAddr, policy: &DiversityPolicy) -> SubnetKey {
    match address.ip() {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            let mask = policy.ipv4_prefix.min(32) as u32;
            let mut value = u32::from_be_bytes(octets);
            if mask > 0 {
                value &= !((1u32 << (32 - mask)) - 1);
            } else {
                value = 0;
            }
            SubnetKey::V4(value.to_be_bytes())
        }
        IpAddr::V6(v6) => {
            let segments = v6.segments();
            let mut bytes = [0u8; 16];
            for (index, segment) in segments.iter().enumerate() {
                bytes[index * 2] = (segment >> 8) as u8;
                bytes[index * 2 + 1] = *segment as u8;
            }
            let bits = policy.ipv6_prefix.min(128);
            let full = (bits / 8) as usize;
            let rem = bits % 8;
            if full < 16 {
                for byte in bytes.iter_mut().skip(full) {
                    *byte = 0;
                }
                if rem > 0 && full < 16 {
                    bytes[full] &= 0xffu8 << (8 - rem);
                }
            }
            SubnetKey::V6(bytes)
        }
    }
}

/// Tracks how many peers are known in each subnet.
#[derive(Clone, Debug)]
pub struct PeerDiversity {
    policy: DiversityPolicy,
    subnets: HashMap<SubnetKey, usize>,
}

impl PeerDiversity {
    pub fn new(policy: DiversityPolicy) -> Result<Self> {
        policy.validate()?;
        Ok(Self {
            policy,
            subnets: HashMap::new(),
        })
    }

    pub fn policy(&self) -> &DiversityPolicy {
        &self.policy
    }

    pub fn count(&self, address: &SocketAddr) -> usize {
        self.subnets
            .get(&subnet_key(address, &self.policy))
            .copied()
            .unwrap_or(0)
    }

    pub fn can_add(&self, address: &SocketAddr) -> bool {
        self.count(address) < self.policy.max_per_subnet
    }

    /// Record a peer. Returns `false` when the subnet is full.
    pub fn record(&mut self, address: &SocketAddr) -> bool {
        if !self.can_add(address) {
            return false;
        }
        *self
            .subnets
            .entry(subnet_key(address, &self.policy))
            .or_insert(0) += 1;
        true
    }

    pub fn remove(&mut self, address: &SocketAddr) {
        let key = subnet_key(address, &self.policy);
        if let Some(count) = self.subnets.get_mut(&key) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.subnets.remove(&key);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.subnets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.subnets.is_empty()
    }
}

/// A Kademlia-style routing table plus the addresses needed to dial it.
#[derive(Clone, Debug)]
pub struct DhtTable {
    local: PeerId,
    routing: RoutingTable,
    addresses: HashMap<PeerId, SocketAddr>,
    last_seen: HashMap<PeerId, u64>,
}

impl DhtTable {
    pub fn new(local: PeerId, k: usize) -> Result<Self> {
        Ok(Self {
            local,
            routing: RoutingTable::new(local, k)?,
            addresses: HashMap::new(),
            last_seen: HashMap::new(),
        })
    }

    pub fn local(&self) -> PeerId {
        self.local
    }

    pub fn len(&self) -> usize {
        self.routing.len()
    }

    pub fn is_empty(&self) -> bool {
        self.routing.is_empty()
    }

    /// Insert a peer. Returns `true` when the peer was added or refreshed.
    pub fn add(&mut self, peer: PeerAddress, now_unix: u64) -> bool {
        if peer.peer_id == self.local || peer.peer_id.0 == [0u8; 32] {
            return false;
        }
        let added = self.routing.add(peer.peer_id);
        self.addresses.insert(peer.peer_id, peer.address);
        self.last_seen.insert(peer.peer_id, now_unix);
        added
    }

    /// Insert only when the address passes the policy.
    pub fn try_add(
        &mut self,
        peer: PeerAddress,
        now_unix: u64,
        local: Option<SocketAddr>,
        policy: &AddressPolicy,
    ) -> Result<bool> {
        validate_peer_address(peer.address, local, policy)?;
        Ok(self.add(peer, now_unix))
    }

    pub fn remove(&mut self, peer: &PeerId) {
        self.routing.remove(peer);
        self.addresses.remove(peer);
        self.last_seen.remove(peer);
    }

    pub fn contains(&self, peer: &PeerId) -> bool {
        self.routing.contains(peer)
    }

    pub fn address(&self, peer: &PeerId) -> Option<SocketAddr> {
        self.addresses.get(peer).copied()
    }

    pub fn last_seen(&self, peer: &PeerId) -> Option<u64> {
        self.last_seen.get(peer).copied()
    }

    /// Closest peers to `target`, ascending XOR distance.
    pub fn closest(&self, target: &PeerId, limit: usize) -> Vec<PeerAddress> {
        self.routing
            .closest(target, limit)
            .into_iter()
            .filter_map(|peer| {
                self.addresses
                    .get(&peer)
                    .map(|address| PeerAddress {
                        peer_id: peer,
                        address: *address,
                    })
            })
            .collect()
    }

    /// Merge a peer-exchange answer, validating every address. Returns the
    /// number of newly added peers.
    pub fn apply_exchange(
        &mut self,
        peers: &[PeerAddress],
        now_unix: u64,
        local: Option<SocketAddr>,
        policy: &AddressPolicy,
    ) -> Result<usize> {
        let mut added = 0;
        for peer in peers.iter().take(MAX_EXCHANGE_PEERS) {
            if validate_peer_address(peer.address, local, policy).is_err() {
                continue;
            }
            if self.add(peer.clone(), now_unix) {
                added += 1;
            }
        }
        Ok(added)
    }
}

/// Iterative Kademlia lookup state.
///
/// The caller drives the lookup: it takes [`LookupState::next_candidates`],
/// sends a `FindNode` message to each candidate, and feeds the answers back
/// with [`LookupState::add_results`]. The lookup is complete when the closest
/// known peers have all been queried.
#[derive(Clone, Debug)]
pub struct LookupState {
    target: PeerId,
    visited: BTreeSet<PeerId>,
    queried: BTreeSet<PeerId>,
    results: BTreeSet<PeerId>,
    deadline: Instant,
    parallelism: usize,
}

impl LookupState {
    pub fn new(target: PeerId, timeout: Duration, parallelism: usize) -> Self {
        Self {
            target,
            visited: BTreeSet::new(),
            queried: BTreeSet::new(),
            results: BTreeSet::new(),
            deadline: Instant::now() + timeout,
            parallelism: parallelism.max(1),
        }
    }

    pub fn target(&self) -> PeerId {
        self.target
    }

    pub fn is_expired_at(&self, now: Instant) -> bool {
        now >= self.deadline
    }

    pub fn visited(&self) -> &BTreeSet<PeerId> {
        &self.visited
    }

    pub fn queried(&self) -> &BTreeSet<PeerId> {
        &self.queried
    }

    pub fn results(&self) -> &BTreeSet<PeerId> {
        &self.results
    }

    /// Closest peers that have not been queried yet.
    pub fn next_candidates(&mut self, table: &DhtTable) -> Vec<PeerAddress> {
        let candidates = table.closest(&self.target, table.len());
        let mut out = Vec::new();
        for peer in candidates {
            if self.visited.contains(&peer.peer_id) || self.queried.contains(&peer.peer_id) {
                continue;
            }
            self.queried.insert(peer.peer_id);
            self.visited.insert(peer.peer_id);
            out.push(peer);
            if out.len() >= self.parallelism {
                break;
            }
        }
        out
    }

    /// Merge an answer into the table and the result set.
    pub fn add_results(&mut self, peers: Vec<PeerAddress>, table: &mut DhtTable, now_unix: u64) {
        for peer in peers {
            table.add(peer, now_unix);
            self.results.insert(peer.peer_id);
            self.visited.insert(peer.peer_id);
        }
    }

    /// Whether every peer currently closest to the target has been queried.
    pub fn is_complete(&self, table: &DhtTable, k: usize) -> bool {
        if table.is_empty() {
            return true;
        }
        let closest = table.closest(&self.target, k);
        if closest.is_empty() {
            return true;
        }
        closest
            .iter()
            .all(|peer| self.queried.contains(&peer.peer_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(port: u16) -> SocketAddr {
        format!("127.0.0.1:{port}").parse().unwrap()
    }

    fn peer(seed: u8) -> PeerId {
        PeerId([seed; 32])
    }

    fn address(peer: PeerId, port: u16) -> PeerAddress {
        PeerAddress {
            peer_id: peer,
            address: addr(port),
        }
    }

    #[test]
    fn strict_policy_rejects_private_and_loopback() {
        let policy = AddressPolicy::default();
        assert!(validate_peer_address(addr(29798), None, &policy).is_err());
        assert!(validate_peer_address(
            "10.0.0.1:29798".parse().unwrap(),
            None,
            &policy
        )
        .is_err());
        assert!(validate_peer_address(
            "8.8.8.8:29798".parse().unwrap(),
            None,
            &policy
        )
        .is_ok());
        assert!(validate_peer_address(
            "8.8.8.8:0".parse().unwrap(),
            None,
            &policy
        )
        .is_err());
        assert!(validate_peer_address(
            "8.8.8.8:29798".parse().unwrap(),
            Some("8.8.8.8:29798".parse().unwrap()),
            &policy
        )
        .is_err());
    }

    #[test]
    fn local_testing_policy_allows_loopback() {
        let policy = AddressPolicy::local_testing();
        assert!(validate_peer_address(addr(29798), None, &policy).is_ok());
    }

    #[test]
    fn diversity_caps_one_subnet() {
        let mut diversity = PeerDiversity::new(DiversityPolicy {
            max_per_subnet: 2,
            ..DiversityPolicy::default()
        })
        .unwrap();
        assert!(diversity.record(&addr(1)));
        assert!(diversity.record(&addr(2)));
        assert!(!diversity.record(&addr(3)));
        assert_eq!(diversity.count(&addr(4)), 2);
        diversity.remove(&addr(1));
        assert!(diversity.record(&addr(3)));
    }

    #[test]
    fn dht_table_merges_and_validates_exchange() {
        let local = PeerId([0u8; 32]);
        let mut table = DhtTable::new(local, 4).unwrap();
        let added = table
            .apply_exchange(
                &[address(peer(1), 29798), address(peer(2), 29799)],
                100,
                Some(addr(29798)),
                &AddressPolicy::local_testing(),
            )
            .unwrap();
        assert_eq!(added, 2);
        // A loopback address equal to the local address is rejected.
        assert!(table
            .try_add(address(peer(3), 29798), 101, Some(addr(29798)), &AddressPolicy::local_testing())
            .is_err());
        assert_eq!(table.len(), 2);
        assert!(table.address(&peer(1)).is_some());
        assert_eq!(table.closest(&peer(2), 10).len(), 2);
    }

    #[test]
    fn lookup_visits_closest_peers_and_completes() {
        let local = PeerId([0u8; 32]);
        let mut table = DhtTable::new(local, 4).unwrap();
        for seed in 1..=8u8 {
            table.add(address(peer(seed), 30_000 + seed as u16), 0);
        }
        let target = peer(200);
        let mut lookup = LookupState::new(target, Duration::from_secs(5), 2);
        let mut rounds = 0;
        loop {
            let candidates = lookup.next_candidates(&table);
            if candidates.is_empty() {
                break;
            }
            assert!(rounds < 10);
            rounds += 1;
            let answers: Vec<PeerAddress> = candidates
                .iter()
                .map(|peer| address(peer.peer_id, peer.address.port()))
                .collect();
            lookup.add_results(answers, &mut table, 1);
            if lookup.is_complete(&table, 4) {
                break;
            }
        }
        assert!(lookup.is_complete(&table, 4));
        assert!(lookup.queried().len() >= 2);
    }

    #[test]
    fn lookup_expires() {
        let lookup = LookupState::new(peer(9), Duration::from_millis(1), 1);
        std::thread::sleep(Duration::from_millis(2));
        assert!(lookup.is_expired_at(Instant::now()));
    }
}
