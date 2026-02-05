mod common;

use common::{edge, mask_from_spqr, mk_edges, truth_addability_mask};
use std::collections::HashSet;

use graphum::spqr::PlanarSubgraph;

fn add_k4(e: &mut HashSet<graphum::Edge>, a: usize, b: usize, c: usize, d: usize) {
    let mut add = |u: usize, v: usize| {
        e.insert(edge(u, v));
    };
    add(a, b);
    add(a, c);
    add(a, d);
    add(b, c);
    add(b, d);
    add(c, d);
}

fn run_chain_test() {
    let p = 0;
    let q = 1;
    let r = 2;
    let s = 3;
    let a1 = 4;
    let b1 = 5;
    let a3 = 6;
    let b3 = 7;
    let a4 = 8;
    let b4 = 9;
    let n = 10usize;

    let mut e = HashSet::new();
    add_k4(&mut e, p, q, r, s);
    add_k4(&mut e, p, q, a1, b1);
    add_k4(&mut e, q, r, a3, b3);
    add_k4(&mut e, r, a3, a4, b4);

    let mut pairs: Vec<_> = e.iter().copied().collect();
    pairs.sort_by_key(|edge| (edge.u, edge.v));
    let uv = edge(a1, a4);
    let uv_idx = pairs.len();
    pairs.push(uv);

    let edges_all = mk_edges(&pairs);
    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);
    for i in 0..edges_all.len() {
        if i == uv_idx {
            continue;
        }
        ps.set(i, true);
    }

    let spqr_mask = mask_from_spqr(&ps);
    let selected: HashSet<usize> = (0..edges_all.len()).filter(|i| *i != uv_idx).collect();
    let truth_mask = truth_addability_mask(n, &selected, &pairs);
    common::compare_masks(&spqr_mask, &truth_mask, &selected);
}

#[test]
fn spqr_k4_chain_full_mask_matches_networkx() {
    run_chain_test();
}

#[test]
fn spqr_k4_chain_full_mask_matches_networkx_v2() {
    // variant identical for our purposes
    run_chain_test();
}
