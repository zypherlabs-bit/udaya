/// Udaya DNS Seed Nodes
///
/// Production DNS seed infrastructure for peer discovery.
/// Three geographic regions ensure global coverage and redundancy.
///
/// ## Seed Nodes
/// - `seed-us.Udaya.org` - North America (US East/West)
/// - `seed-eu.Udaya.org` - Europe (EU West/Central)
/// - `seed-ap.Udaya.org` - Asia Pacific (AP Southeast/Northeast)
///
/// ## DNS Records
/// ```text
/// ; A Records (IPv4)
/// seed-us.Udaya.org.    IN A      <production-ip>
/// seed-eu.Udaya.org.    IN A      <production-ip>
/// seed-ap.Udaya.org.    IN A      <production-ip>
///
/// ; AAAA Records (IPv6)
/// seed-us.Udaya.org.    IN AAAA   <production-ipv6>
/// seed-eu.Udaya.org.    IN AAAA   <production-ipv6>
/// seed-ap.Udaya.org.    IN AAAA   <production-ipv6>
///
/// ; SRV Records for P2P service discovery
/// _Udaya-p2p._tcp.seed-us.Udaya.org.  IN SRV 10 10 9798 seed-us.Udaya.org.
/// _Udaya-p2p._tcp.seed-eu.Udaya.org.  IN SRV 10 10 9798 seed-eu.Udaya.org.
/// _Udaya-p2p._tcp.seed-ap.Udaya.org.  IN SRV 10 10 9798 seed-ap.Udaya.org.
///
/// ; TXT Records for version verification
/// seed-us.Udaya.org.    IN TXT    "v=UDYA1;net=mainnet"
/// seed-eu.Udaya.org.    IN TXT    "v=UDYA1;net=mainnet"  
/// seed-ap.Udaya.org.    IN TXT    "v=UDYA1;net=mainnet"
/// ```
///
/// ## Bootstrap Flow
/// 1. Fresh node starts with empty peer list
/// 2. Node queries DNS for seed-us.Udaya.org, seed-eu.Udaya.org, seed-ap.Udaya.org
/// 3. DNS returns A/AAAA records of seed node IPs
/// 4. Node connects to seed nodes via P2P port 9798
/// 5. Seed nodes respond with peer lists via `getaddr`/`addr` messages
/// 6. Node establishes connections to discovered peers
/// 7. Node begins blockchain synchronization
///
/// ## Redundancy
/// - Three geographically diverse seeds
/// - Each seed maintains 1000+ peer connections
/// - Seeds provide rotating peer lists for diversity
/// - If one region fails, others compensate

/// The three official Udaya DNS seed domains
pub const DNS_SEED_DOMAINS: [&str; 3] = [
    "seed-us.Udaya.org",
    "seed-eu.Udaya.org",
    "seed-ap.Udaya.org",
];

/// Default P2P port for Udaya
pub const UDAYA_P2P_PORT: u16 = 9798;

/// DNS seed configuration with health check parameters
#[derive(Debug, Clone)]
pub struct DnsSeedConfig {
    pub domain: String,
    pub port: u16,
    pub ipv4_priority: u8,
    pub ipv6_support: bool,
    pub health_check_interval_secs: u64,
    pub max_peers_per_seed: usize,
}

impl DnsSeedConfig {
    pub fn new(domain: &str) -> Self {
        Self {
            domain: domain.to_string(),
            port: UDAYA_P2P_PORT,
            ipv4_priority: 10,
            ipv6_support: true,
            health_check_interval_secs: 300,
            max_peers_per_seed: 1000,
        }
    }

    /// Get the full address string for DNS resolution
    pub fn address(&self) -> String {
        format!("{}:{}", self.domain, self.port)
    }
}

/// Get the default DNS seed configurations for all regions
pub fn get_default_seeds() -> Vec<DnsSeedConfig> {
    DNS_SEED_DOMAINS
        .iter()
        .map(|d| DnsSeedConfig::new(d))
        .collect()
}

/// Verify DNS seed configuration
pub fn verify_seed_configuration() -> Vec<(String, bool, String)> {
    let seeds = get_default_seeds();
    seeds
        .iter()
        .map(|seed| {
            let addr = seed.address();
            // Validate format: domain:port
            let valid = !seed.domain.is_empty() && seed.port > 0 && seed.domain.contains('.');
            if valid {
                (
                    seed.domain.clone(),
                    true,
                    format!("Configured on port {}", seed.port),
                )
            } else {
                (
                    seed.domain.clone(),
                    false,
                    "Invalid configuration".to_string(),
                )
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_seed_domains_count() {
        assert_eq!(DNS_SEED_DOMAINS.len(), 3);
    }

    #[test]
    fn test_dns_seed_domain_format() {
        for domain in &DNS_SEED_DOMAINS {
            assert!(domain.starts_with("seed-"));
            assert!(domain.ends_with(".Udaya.org"));
            assert!(domain.contains('-'));
        }
    }

    #[test]
    fn test_dns_seed_config() {
        let config = DnsSeedConfig::new("seed-us.Udaya.org");
        assert_eq!(config.port, 9798);
        assert!(config.address().contains(":9798"));
        assert_eq!(config.max_peers_per_seed, 1000);
    }

    #[test]
    fn test_default_seeds() {
        let seeds = get_default_seeds();
        assert_eq!(seeds.len(), 3);
        for seed in &seeds {
            assert!(seed.domain.contains("Udaya.org"));
        }
    }

    #[test]
    fn test_verify_seed_configuration() {
        let results = verify_seed_configuration();
        assert_eq!(results.len(), 3);
        for (domain, valid, msg) in &results {
            assert!(valid, "Seed {} should be valid: {}", domain, msg);
            assert!(domain.contains("Udaya.org"));
        }
    }

    #[test]
    fn test_port_constant() {
        assert_eq!(UDAYA_P2P_PORT, 9798);
    }
}
