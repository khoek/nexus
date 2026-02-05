mod common;

use common::{edge, edges_complete, mask_from_spqr, mk_edges, pair_index, truth_addability_mask};
use std::collections::HashSet;

use graphum::spqr::PlanarSubgraph;

fn ladder_edges(k: usize) -> Vec<graphum::Edge> {
    let mut e = Vec::new();
    for i in 0..k {
        e.push(edge(i, k + i));
        if i + 1 < k {
            e.push(edge(i, i + 1));
            e.push(edge(k + i, k + i + 1));
        }
    }
    e
}

fn prism_edges(m: usize) -> Vec<graphum::Edge> {
    let mut e = Vec::new();
    for i in 0..m {
        let next = (i + 1) % m;
        e.push(edge(i, next));
        e.push(edge(m + i, m + next));
        e.push(edge(i, m + i));
    }
    e
}

#[test]
fn ladder_family_truth_mask_matches_spqr() {
    for k in [3usize, 5, 7] {
        let n = 2 * k;
        let edges = edges_complete(n);
        let edges_all = mk_edges(&edges);
        let idx = pair_index(&edges);
        let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);

        let e = ladder_edges(k);
        let selected: HashSet<usize> = e.iter().map(|edge| idx[edge]).collect();
        for i in &selected {
            ps.set(*i, true);
        }

        let mv = mask_from_spqr(&ps);
        let truth = truth_addability_mask(n, &selected, &edges);
        for (i, (a, b)) in mv.iter().zip(truth.iter()).enumerate() {
            if selected.contains(&i) {
                continue;
            }
            assert_eq!(*a as i32, *b as i32);
        }
    }
}

#[test]
fn prism_family_truth_mask_matches_spqr() {
    for m in [3usize, 4, 5] {
        let n = 2 * m;
        let edges = edges_complete(n);
        let edges_all = mk_edges(&edges);
        let idx = pair_index(&edges);
        let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);

        let e = prism_edges(m);
        let selected: HashSet<usize> = e.iter().map(|edge| idx[edge]).collect();
        for i in &selected {
            ps.set(*i, true);
        }

        let mv = mask_from_spqr(&ps);
        let truth = truth_addability_mask(n, &selected, &edges);
        for (i, (a, b)) in mv.iter().zip(truth.iter()).enumerate() {
            if selected.contains(&i) {
                continue;
            }
            assert_eq!(*a as i32, *b as i32);
        }
    }
}
