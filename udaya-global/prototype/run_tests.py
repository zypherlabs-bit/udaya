import sys, os
sys.path.insert(0, os.path.dirname(__file__))

def run():
    from uadc_core import (
        GENESIS, QC, Block, NodeState, quorum, safe_node, extends,
        ensure_not_protected_magic, PROTECTED_MAINNET_NETWORK_MAGIC,
        EXPERIMENTAL_NETWORK_MAGIC, EXPERIMENTAL_CHAIN_ID,
    )
    results = []

    def check(name, cond, detail=""):
        results.append((name, bool(cond), detail))
        if not cond:
            raise AssertionError(f"FAIL {name}: {detail}")

    # isolation
    try:
        ensure_not_protected_magic(PROTECTED_MAINNET_NETWORK_MAGIC)
        check("rejects_protected_mainnet_magic", False)
    except Exception:
        check("rejects_protected_mainnet_magic", True)
    ensure_not_protected_magic(EXPERIMENTAL_NETWORK_MAGIC)
    check("accepts_experimental_magic", True)
    check("chain_id_distinct", EXPERIMENTAL_CHAIN_ID == "uadc-experimental-0001")

    # quorum
    check("quorum_n4", quorum(4) == 3)
    check("quorum_n7", quorum(7) == 5)

    # basic commit (2-chain) happy path, n=4
    n = 4
    nodes = [NodeState(i, n) for i in range(n)]
    genesis_qc = nodes[0].high_qc
    b1 = Block("B1", GENESIS, 1, 0, 0, genesis_qc)
    for nd in nodes: nd.observe_block(b1)
    # prepare votes -> QC1(B1,0)
    for nd in nodes: assert nd.vote_prepare(b1) == "B1"
    qc1_b1 = QC("B1", 0, "PREPARE", (0,1,2))
    for nd in nodes:
        nd.observe_qc(qc1_b1)
        assert nd.locked_qc.rank() >= qc1_b1.rank(), "lock must update on QC1"
        nd.vote_precommit(b1)
    qc2_b1 = QC("B1", 0, "PRECOMMIT", (0,1,2))
    b2 = Block("B2", "B1", 2, 1, 1, qc1_b1)
    qc1_b2 = QC("B2", 1, "PREPARE", (0,1,2))
    for nd in nodes:
        nd.observe_qc(qc2_b1)
        nd.observe_block(b2)
        nd.observe_qc(qc1_b2)
        newly = nd.try_commit()
    for nd in nodes:
        check(f"commit_n4_node{nd.vid}", nd.committed[:2] == [GENESIS, "B1"], str(nd.committed))

    # safe_node: cannot vote for conflicting block below lock without higher justify
    nd = NodeState(0, 4)
    # lock on B1 chain
    nd.observe_block(b1)
    nd.observe_qc(qc1_b1)
    assert nd.locked_qc.block == "B1"
    # conflicting B1x at same height/round different parent path not extending B1, justify at genesis
    b1x = Block("B1x", GENESIS, 1, 2, 2, QC(GENESIS,-1,"PRECOMMIT",tuple()))
    nd.observe_block(b1x)
    check("safe_node_rejects_conflict_below_lock", not safe_node(nd.blocks, b1x, nd.locked_qc))
    check("cannot_prepare_conflict_below_lock", nd.vote_prepare(b1x) is None)
    # higher justify allows
    b1y = Block("B1y", GENESIS, 1, 3, 3, QC("X", 5, "PREPARE", (0,1,2)))
    nd.blocks["X"] = Block("X", GENESIS, 1, 5, 0, QC(GENESIS,-1,"PRECOMMIT",tuple()))
    nd.observe_block(b1y)
    check("safe_node_allows_higher_justify", safe_node(nd.blocks, b1y, nd.locked_qc))

    # vote uniqueness
    nd2 = NodeState(1, 4)
    nd2.observe_block(b1)
    assert nd2.vote_prepare(b1) == "B1"
    check("vote_uniqueness_prepare", nd2.vote_prepare(b1) is None)

    passed = sum(1 for _,ok,_ in results if ok)
    return passed, len(results), results

if __name__ == "__main__":
    passed, total, results = run()
    for name, ok, detail in results:
        print(f"{'PASS' if ok else 'FAIL'} {name} {detail}".rstrip())
    print(f"SUMMARY {passed}/{total} passed")
    if passed != total:
        sys.exit(1)
