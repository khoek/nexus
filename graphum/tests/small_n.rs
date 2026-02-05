mod common;

use common::{edges_complete, mask_from_spqr, mk_edges, truth_addability_mask};
use std::collections::HashSet;

use graphum::spqr::PlanarSubgraph;

#[test]
fn n_0_and_1_have_no_candidates() {
    for n in [0, 1] {
        let edges = edges_complete(n);
        let ea = mk_edges(&edges);
        let ps = PlanarSubgraph::new(n, &ea, &vec![false; ea.len()]);
        let mv = mask_from_spqr(&ps);
        assert!(mv.is_empty());
    }
}

#[test]
fn n_2_single_edge_roundtrip() {
    let n = 2;
    let edges = edges_complete(n);
    let ea = mk_edges(&edges);
    let mut ps = PlanarSubgraph::new(n, &ea, &vec![false; ea.len()]);
    let mv0 = mask_from_spqr(&ps);
    assert_eq!(mv0, vec![true]);

    ps.set(0, true);
    let mv1 = mask_from_spqr(&ps);
    assert_eq!(mv1, vec![false]);

    ps.set(0, false);
    let mv2 = mask_from_spqr(&ps);
    let truth = truth_addability_mask(n, &HashSet::new(), &edges);
    assert_eq!(mv2, truth);
    assert_eq!(mv2, vec![true]);
}

#[test]
fn n_3_triangle_is_maximal_planar() {
    let n = 3;
    let edges = edges_complete(n);
    let ea = mk_edges(&edges);
    let mut ps = PlanarSubgraph::new(n, &ea, &vec![false; ea.len()]);
    for i in 0..edges.len() {
        ps.set(i, true);
    }

    let mv = mask_from_spqr(&ps);
    assert!(mv.iter().all(|x| !*x));

    let truth = truth_addability_mask(n, &(0..edges.len()).collect(), &edges);
    assert!(truth.iter().all(|x| !*x));
}
