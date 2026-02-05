mod common;

use common::{edge, edges_complete, mask_from_spqr, mk_edges, pair_index, truth_addability_mask};
use std::collections::HashSet;

use graphum::spqr::PlanarSubgraph;

#[test]
fn k5_subdivision_minus_one_edge_blocked() {
    let n = 9;
    let edges = edges_complete(n);
    let edges_all = mk_edges(&edges);
    let idx = pair_index(&edges);
    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);

    let subdiv: std::collections::HashMap<graphum::Edge, usize> =
        [((0, 2), 5usize), ((0, 3), 6), ((1, 3), 7), ((2, 4), 8)]
            .iter()
            .copied()
            .map(|((u, v), w)| (edge(u, v), w))
            .collect();
    let add_path = |u: usize, v: usize, ps: &mut PlanarSubgraph| {
        if let Some(&w) = subdiv.get(&edge(u, v)) {
            ps.set(idx[&edge(u, w)], true);
            ps.set(idx[&edge(v, w)], true);
        } else {
            ps.set(idx[&edge(u, v)], true);
        }
    };

    let missing = (0, 1);
    for u in 0..5 {
        for v in (u + 1)..5 {
            if (u, v) == missing {
                continue;
            }
            add_path(u, v, &mut ps);
        }
    }

    let mv = mask_from_spqr(&ps);
    let add_idx = idx[&edge(missing.0, missing.1)];
    assert!(!mv[add_idx]);

    let selected: HashSet<usize> = idx.values().copied().filter(|i| *i != add_idx).collect();
    let truth = truth_addability_mask(n, &selected, &edges);
    assert!(!truth[add_idx]);
}

#[test]
fn k33_subdivision_minus_one_edge_blocked() {
    let n = 10;
    let edges = edges_complete(n);
    let edges_all = mk_edges(&edges);
    let idx = pair_index(&edges);
    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);

    let subdiv: std::collections::HashMap<graphum::Edge, usize> =
        [((0, 4), 6usize), ((1, 5), 7), ((2, 3), 8), ((2, 5), 9)]
            .iter()
            .copied()
            .map(|((u, v), w)| (edge(u, v), w))
            .collect();
    let add_path = |u: usize, v: usize, ps: &mut PlanarSubgraph| {
        if let Some(&w) = subdiv.get(&edge(u, v)) {
            ps.set(idx[&edge(u, w)], true);
            ps.set(idx[&edge(v, w)], true);
        } else {
            ps.set(idx[&edge(u, v)], true);
        }
    };

    let missing = (0, 3);
    let a = [0, 1, 2];
    let b = [3, 4, 5];
    for &u in &a {
        for &v in &b {
            if (u, v) == missing {
                continue;
            }
            add_path(u, v, &mut ps);
        }
    }

    let mv = mask_from_spqr(&ps);
    let add_idx = idx[&edge(missing.0, missing.1)];
    assert!(!mv[add_idx]);

    let selected: HashSet<usize> = idx.values().copied().filter(|i| *i != add_idx).collect();
    let truth = truth_addability_mask(n, &selected, &edges);
    assert!(!truth[add_idx]);
}
