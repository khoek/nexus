mod common;

use common::{edge, edges_complete, mk_edges, planarize_edges_greedy, truth_addability_mask};
use std::collections::HashSet;

use graphum::spqr::PlanarSubgraph;

#[test]
fn spqr_addability_matches_networkx_on_incremental_planar_build() {
    let n = 10;
    let edges = edges_complete(n);
    let edges_all = mk_edges(&edges);

    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);
    let tri1 = [edge(0, 1), edge(1, 2), edge(2, 0)];
    let tri2 = [edge(5, 6), edge(6, 7), edge(7, 5)];
    let bridge = [edge(2, 5)];

    let pair2idx = common::pair_index(&edges);
    let mut h_idx = Vec::new();
    for e in tri1.iter().chain(tri2.iter()).chain(bridge.iter()) {
        let k = pair2idx[e];
        ps.set(k, true);
        h_idx.push(k);
    }

    let spqr_mask = common::mask_from_spqr(&ps);
    let truth_mask = truth_addability_mask(n, &h_idx.iter().copied().collect(), &edges);
    let ignore: HashSet<usize> = h_idx.into_iter().collect();
    common::compare_masks(&spqr_mask, &truth_mask, &ignore);
}

#[test]
fn spqr_dynamic_add_remove_consistency_with_greedy() {
    let n = 12;
    let edges = edges_complete(n);
    let edges_all = mk_edges(&edges);

    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);

    let mut keep_idx = planarize_edges_greedy(n, &edges);
    keep_idx.sort_unstable();

    let mut selected: HashSet<usize> = HashSet::new();
    for &i in &keep_idx {
        ps.set(i, true);
        selected.insert(i);
    }

    let spqr_mask = common::mask_from_spqr(&ps);
    let truth_mask = truth_addability_mask(n, &selected, &edges);
    common::compare_masks(&spqr_mask, &truth_mask, &selected);

    let remove_count = (keep_idx.len() / 8).clamp(1, 4);
    let to_remove = keep_idx
        .iter()
        .take(remove_count)
        .copied()
        .collect::<Vec<_>>();
    for ridx in to_remove {
        ps.set(ridx, false);
        selected.remove(&ridx);
        let spqr_mask = common::mask_from_spqr(&ps);
        let truth_mask = truth_addability_mask(n, &selected, &edges);
        common::compare_masks(&spqr_mask, &truth_mask, &selected);
    }

    loop {
        let spqr_mask = common::mask_from_spqr(&ps);
        let truth_mask = truth_addability_mask(n, &selected, &edges);
        let candidates: Vec<usize> = (0..edges.len())
            .filter(|i| !selected.contains(i) && spqr_mask[*i] && truth_mask[*i])
            .collect();
        if candidates.is_empty() {
            break;
        }
        let i = candidates[0];
        ps.set(i, true);
        selected.insert(i);
        let spqr_mask = common::mask_from_spqr(&ps);
        let truth_mask = truth_addability_mask(n, &selected, &edges);
        common::compare_masks(&spqr_mask, &truth_mask, &selected);
    }
}

#[test]
fn spqr_addability_cross_components_all_addable() {
    let n = 8;
    let edges = edges_complete(n);
    let edges_all = mk_edges(&edges);
    let ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);

    let mask = common::mask_from_spqr(&ps);
    assert_eq!(mask.len(), edges.len());
    assert!(mask.iter().all(|b| *b));
}
