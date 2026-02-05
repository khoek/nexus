mod common;

use common::{
    build_triangulated_polygon_edges, edges_complete, greedily_fill_to_maximal_planar,
    mask_from_spqr, mk_edges, pair_index, truth_addability_mask,
};
use std::collections::HashSet;

use graphum::{Edge, spqr::PlanarSubgraph};

fn indices_of(edges: &[Edge], es: &[Edge]) -> Vec<usize> {
    let idx = pair_index(edges);
    es.iter().map(|edge| idx[edge]).collect()
}

#[test]
fn fill_from_triangulated_polygon_reaches_maximal_planar() {
    let n = 10;
    let edges = edges_complete(n);
    let edges_all = mk_edges(&edges);

    let base_pairs = build_triangulated_polygon_edges(n);
    let base_idx: HashSet<usize> = indices_of(&edges, &base_pairs).into_iter().collect();
    let mut base_idx_pruned = base_idx.clone();
    assert!(
        base_idx.len() >= 3,
        "cannot prune below zero vertices in maximal planar test"
    );
    while base_idx_pruned.len() > base_idx.len() - 3 {
        if let Some(x) = base_idx_pruned.iter().next().cloned() {
            base_idx_pruned.remove(&x);
        } else {
            break;
        }
    }

    let sel = greedily_fill_to_maximal_planar(n, &edges, base_idx_pruned.clone());
    assert_eq!(sel.len(), 3 * n - 6);

    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);
    for i in sel.iter().copied() {
        ps.set(i, true);
    }
    let mask = mask_from_spqr(&ps);
    assert!(
        mask.iter()
            .enumerate()
            .all(|(i, b)| sel.contains(&i) || !*b)
    );

    let truth = truth_addability_mask(n, &sel, &edges);
    assert!(truth.iter().all(|x| !*x));
}

#[test]
fn fill_from_two_components_reaches_global_maximal_planar() {
    let n = 12;
    let edges = edges_complete(n);
    let edges_all = mk_edges(&edges);

    let tri_a = build_triangulated_polygon_edges(6);
    let tri_b: Vec<_> = build_triangulated_polygon_edges(6)
        .into_iter()
        .map(|Edge { u, v }| Edge { u: u + 6, v: v + 6 })
        .collect();
    let base_idx: HashSet<usize> =
        indices_of(&edges, &[tri_a.as_slice(), tri_b.as_slice()].concat())
            .into_iter()
            .collect();

    let sel = greedily_fill_to_maximal_planar(n, &edges, base_idx);
    assert_eq!(sel.len(), 3 * n - 6);

    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);
    for i in sel.iter().copied() {
        ps.set(i, true);
    }
    let mask = mask_from_spqr(&ps);
    assert!(
        mask.iter()
            .enumerate()
            .all(|(i, b)| sel.contains(&i) || !*b)
    );

    let truth = truth_addability_mask(n, &sel, &edges);
    assert!(truth.iter().all(|x| !*x));
}
