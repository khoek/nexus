mod common;

use common::{edges_complete, mask_from_spqr, mk_edges};

use graphum::spqr::PlanarSubgraph;

#[test]
fn query_returns_list_and_reflects_updates() {
    let n = 10;
    let edges = edges_complete(n);
    let edges_all = mk_edges(&edges);
    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);

    let mv = ps.query();
    assert_eq!(mv.len(), edges.len());

    let i = mv
        .iter()
        .enumerate()
        .find(|(_, b)| **b)
        .map(|(idx, _)| idx)
        .expect("expected at least one addable index");
    let before = mv[i];
    assert!(before);
    ps.set(i, true);

    assert_eq!(mv[i], before);
    let mv_after_add = ps.query();
    assert!(!mv_after_add[i]);

    ps.set(i, true);
    assert_eq!(mv[i], before);
    let mv_after_readd = ps.query();
    assert!(!mv_after_readd[i]);

    ps.set(i, false);
    assert_eq!(mv[i], before);
    let mv_after_remove = ps.query();
    assert!(mv_after_remove[i]);
}

#[test]
fn selected_candidates_always_zero_in_mask() {
    let n = 9;
    let edges = edges_complete(n);
    let edges_all = mk_edges(&edges);
    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);

    let mv = mask_from_spqr(&ps);
    let mut chosen = Vec::new();
    for (idx, b) in mv.iter().enumerate() {
        if *b {
            ps.set(idx, true);
            chosen.push(idx);
        }
        if chosen.len() >= 5 {
            break;
        }
    }
    let cur = ps.query();
    assert!(
        chosen.iter().all(|&i| !cur[i]),
        "selected entries must be zero"
    );
}
