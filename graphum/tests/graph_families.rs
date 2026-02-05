mod common;

use common::{edge, edges_complete, mask_from_spqr, mk_edges, pair_index, truth_addability_mask};
use std::collections::HashSet;

use graphum::spqr::PlanarSubgraph;

fn cycle_edges(n: usize) -> Vec<graphum::Edge> {
    (0..n).map(|i| edge(i, (i + 1) % n)).collect()
}

fn grid_edges(w: usize, h: usize) -> Vec<graphum::Edge> {
    let mut e = Vec::new();
    let nid = |x: usize, y: usize| y * w + x;
    for y in 0..h {
        for x in 0..w {
            if x + 1 < w {
                e.push(edge(nid(x, y), nid(x + 1, y)));
            }
            if y + 1 < h {
                e.push(edge(nid(x, y), nid(x, y + 1)));
            }
        }
    }
    e
}

#[test]
fn truth_mask_on_cycle_matches_spqr() {
    for n in [6usize, 8, 10] {
        let edges = edges_complete(n);
        let idx = pair_index(&edges);
        let edges_all = mk_edges(&edges);
        let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);

        let e = cycle_edges(n);
        let selected: HashSet<usize> = e.iter().map(|edge| idx[edge]).collect();
        for i in &selected {
            ps.set(*i, true);
        }

        let mask = mask_from_spqr(&ps);
        let truth = truth_addability_mask(n, &selected, &edges);
        for (i, (a, b)) in mask.iter().zip(truth.iter()).enumerate() {
            if selected.contains(&i) {
                continue;
            }
            assert_eq!(*a as i32, *b as i32);
        }
    }
}

#[test]
fn truth_mask_on_grid_matches_spqr() {
    let w = 3;
    let h = 3;
    let n = w * h;
    let edges = edges_complete(n);
    let idx = pair_index(&edges);
    let edges_all = mk_edges(&edges);
    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);

    let e = grid_edges(w, h);
    let selected: HashSet<usize> = e.iter().map(|edge| idx[edge]).collect();
    for i in &selected {
        ps.set(*i, true);
    }

    let mask = mask_from_spqr(&ps);
    let truth = truth_addability_mask(n, &selected, &edges);
    for (i, (a, b)) in mask.iter().zip(truth.iter()).enumerate() {
        if selected.contains(&i) {
            continue;
        }
        assert_eq!(*a as i32, *b as i32);
    }
}
