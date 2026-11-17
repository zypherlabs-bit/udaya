"""
Spec-traced reference core for candidate U-ADC 2-chain HotStuff-like consensus.
Isolated experimental prototype (Phase 4). NOT audited/proven/production-ready.
Traceability: uadc-research/phase3/UADC_BFT_SPEC_FINALIZED.md, FORMAL_CONSENSUS_MODEL.md.
"""
from __future__ import annotations
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Tuple, Set

# --- Isolation / identity ---
EXPERIMENTAL_NETWORK_NAME = "udaya-global-experimental"
EXPERIMENTAL_CHAIN_ID = "uadc-experimental-0001"
EXPERIMENTAL_NETWORK_MAGIC = bytes([0x55, 0x41, 0x44, 0x43])
PROTECTED_MAINNET_NETWORK_MAGIC = bytes([0xBF, 0x59, 0x1A, 0xE7])
EXPERIMENTAL_DEFAULT_P2P_PORT = 19799
EXPERIMENTAL_DEFAULT_DATADIR = "data/udaya-global-experimental"
GENESIS = "B0"

class IsolationError(ValueError):
    pass

def ensure_not_protected_magic(magic: bytes) -> None:
    if bytes(magic) == PROTECTED_MAINNET_NETWORK_MAGIC:
        raise IsolationError("protected existing Udaya mainnet magic refused")
    if bytes(magic) != EXPERIMENTAL_NETWORK_MAGIC:
        raise IsolationError(f"unknown/non-experimental network magic refused: {bytes(magic)!r}")

@dataclass(frozen=True)
class QC:
    block: str
    round: int
    phase: str  # PREPARE or PRECOMMIT
    signers: Tuple[int, ...]
    def rank(self) -> Tuple[int, int]:
        return (self.round, 1 if self.phase == "PRECOMMIT" else 0)

@dataclass(frozen=True)
class Block:
    bid: str
    parent: str
    height: int
    round: int
    proposer: int
    justify: QC

def quorum(n: int) -> int:
    return (2 * n) // 3 + 1

def extends(blocks: Dict[str, Block], child: str, ancestor: str) -> bool:
    cur = child
    seen = set()
    while cur != ancestor:
        if cur in seen or cur not in blocks:
            return False
        seen.add(cur)
        cur = blocks[cur].parent
    return True

def safe_node(blocks: Dict[str, Block], b: Block, locked_qc: QC) -> bool:
    return extends(blocks, b.bid, locked_qc.block) or b.justify.round > locked_qc.round

@dataclass
class NodeState:
    vid: int
    n: int
    q: int = field(init=False)
    high_qc: QC = field(default_factory=lambda: QC(GENESIS, -1, "PRECOMMIT", tuple()))
    locked_qc: QC = field(default_factory=lambda: QC(GENESIS, -1, "PRECOMMIT", tuple()))
    voted_prepare: Dict[int, str] = field(default_factory=dict)
    voted_precommit: Dict[int, str] = field(default_factory=dict)
    committed: List[str] = field(default_factory=lambda: [GENESIS])
    blocks: Dict[str, Block] = field(default_factory=dict)
    qcs: Dict[Tuple[str, int, str], QC] = field(default_factory=dict)

    def __post_init__(self):
        self.q = quorum(self.n)
        if GENESIS not in self.blocks:
            self.blocks[GENESIS] = Block(GENESIS, GENESIS, 0, -1, -1, QC(GENESIS, -1, "PRECOMMIT", tuple()))

    def observe_block(self, b: Block) -> None:
        self.blocks[b.bid] = b

    def observe_qc(self, qc: QC) -> None:
        self.qcs[(qc.block, qc.round, qc.phase)] = qc
        if qc.rank() > self.high_qc.rank():
            self.high_qc = qc
        if qc.phase == "PREPARE" and qc.rank() > self.locked_qc.rank():
            self.locked_qc = qc

    def can_vote_prepare(self, b: Block) -> bool:
        if b.round in self.voted_prepare:
            return False
        if b.bid not in self.blocks:
            return False
        return safe_node(self.blocks, b, self.locked_qc)

    def vote_prepare(self, b: Block):
        if not self.can_vote_prepare(b):
            return None
        self.voted_prepare[b.round] = b.bid
        return b.bid

    def can_vote_precommit(self, b: Block) -> bool:
        if b.round in self.voted_precommit:
            return False
        return self.qcs.get((b.bid, b.round, "PREPARE")) is not None

    def vote_precommit(self, b: Block):
        if not self.can_vote_precommit(b):
            return None
        self.voted_precommit[b.round] = b.bid
        return b.bid

    def try_commit(self) -> List[str]:
        newly: List[str] = []
        qc1_by_round: Dict[int, List[QC]] = {}
        qc2_blocks: Set[Tuple[str, int]] = set()
        for (bid, r, ph), qc in self.qcs.items():
            if ph == "PREPARE":
                qc1_by_round.setdefault(r, []).append(qc)
            elif ph == "PRECOMMIT":
                qc2_blocks.add((bid, r))
        changed = True
        while changed:
            changed = False
            for (bid, r) in list(qc2_blocks):
                for qc1 in qc1_by_round.get(r + 1, []):
                    child = qc1.block
                    if child in self.blocks and self.blocks[child].parent == bid:
                        chain = []
                        cur = bid
                        while cur != GENESIS and cur not in self.committed:
                            if cur not in self.blocks:
                                chain = []; break
                            chain.append(cur)
                            cur = self.blocks[cur].parent
                        if chain:
                            for b in reversed(chain):
                                if b not in self.committed and b in self.blocks and self.blocks[b].parent == self.committed[-1]:
                                    self.committed.append(b)
                                    newly.append(b)
                                    changed = True
        return newly
