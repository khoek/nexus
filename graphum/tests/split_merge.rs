mod common;

use common::{edge, edges_complete, mk_edges};

use graphum::spqr::PlanarSubgraph;

#[test]
fn remove_edge_that_splits_component_makes_cross_pairs_addable() {
    let n = 6;
    let pairs = edges_complete(n);
    let edges_all = mk_edges(&pairs);
    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);
    let idx = common::pair_index(&pairs);

    let path: [(usize, usize); 5] = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)];
    for &(u, v) in &path {
        ps.set(idx[&edge(u, v)], true);
    }

    ps.set(idx[&edge(2, 3)], false);

    let mv = ps.query();

    let left = [0, 1, 2];
    let right = [3, 4, 5];
    for &u in &left {
        for &v in &right {
            assert!(
                mv[idx[&edge(u, v)]],
                "pair ({u},{v}) should be addable after split"
            );
        }
    }
}
