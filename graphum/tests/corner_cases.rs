mod common;

use common::{edge, edges_complete, mask_from_spqr, mk_edges, pair_index, truth_addability_mask};
use std::collections::{HashMap, HashSet};

use graphum::spqr::PlanarSubgraph;

fn idx_of(idx: &HashMap<graphum::Edge, usize>, u: usize, v: usize) -> usize {
    idx[&edge(u, v)]
}

#[test]
fn k5_minus_one_edge_blocked() {
    let n = 5;
    let edges = edges_complete(n);
    let edges_all = mk_edges(&edges);
    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);

    let idx = pair_index(&edges);
    let missing = (0, 1);
    for u in 0..n {
        for v in (u + 1)..n {
            if (u, v) == missing {
                continue;
            }
            ps.set(idx[&edge(u, v)], true);
        }
    }

    let mv = mask_from_spqr(&ps);
    let add_idx = idx_of(&idx, missing.0, missing.1);
    assert!(!mv[add_idx]);

    let truth = truth_addability_mask(
        n,
        &(idx.values().copied().filter(|i| *i != add_idx).collect()),
        &edges,
    );
    assert!(!truth[add_idx]);
}

#[test]
fn k33_minus_one_edge_blocked() {
    let n = 6;
    let a = [0, 1, 2];
    let b = [3, 4, 5];
    let edges = edges_complete(n);
    let edges_all = mk_edges(&edges);
    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);

    let idx = pair_index(&edges);
    let missing = (1, 3);
    for &u in &a {
        for &v in &b {
            if (u, v) == missing {
                continue;
            }
            let (uu, vv) = if u < v { (u, v) } else { (v, u) };
            ps.set(idx[&edge(uu, vv)], true);
        }
    }

    let mv = mask_from_spqr(&ps);
    let add_idx = idx_of(&idx, missing.0, missing.1);
    assert!(!mv[add_idx]);

    let truth = truth_addability_mask(
        n,
        &(idx.values().copied().filter(|i| *i != add_idx).collect()),
        &edges,
    );
    assert!(!truth[add_idx]);
}

#[test]
fn wheel_graph_is_maximal_planar() {
    let n = 8;
    let edges = edges_complete(n);
    let edges_all = mk_edges(&edges);
    let idx = pair_index(&edges);
    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);

    let mut selected: HashSet<usize> = HashSet::new();
    for i in 1..(n - 1) {
        let eidx = idx[&edge(i, i + 1)];
        ps.set(eidx, true);
        selected.insert(eidx);
    }
    let eidx = idx[&edge(1, n - 1)];
    ps.set(eidx, true);
    selected.insert(eidx);
    for v in 1..n {
        let eidx = idx[&edge(0, v)];
        ps.set(eidx, true);
        selected.insert(eidx);
    }

    let mv = mask_from_spqr(&ps);
    let truth = truth_addability_mask(n, &selected, &edges);
    let non_sel: Vec<_> = (0..mv.len()).filter(|i| !selected.contains(i)).collect();
    assert!(non_sel.iter().any(|i| truth[*i]));
    for i in non_sel {
        assert_eq!(mv[i], truth[i]);
    }
}
