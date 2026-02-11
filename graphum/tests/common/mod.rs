#![allow(dead_code)]

use rand::{RngExt, SeedableRng, rngs::StdRng};
use rustworkx_core::{petgraph::graph::UnGraph, planar::is_planar};
use std::collections::{HashMap, HashSet};

use graphum::{Edge, spqr::PlanarSubgraph};

pub fn rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

pub fn edge(u: usize, v: usize) -> Edge {
    norm(Edge { u, v })
}

fn norm(Edge { u, v }: Edge) -> Edge {
    let (u, v) = if u <= v { (u, v) } else { (v, u) };
    Edge { u, v }
}

pub fn edges_complete(n: usize) -> Vec<Edge> {
    let mut out = Vec::new();
    for u in 0..n {
        for v in (u + 1)..n {
            out.push(Edge { u, v });
        }
    }
    out
}

pub fn edges_random(n: usize, m: usize, rng: &mut StdRng) -> Vec<Edge> {
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(m);
    while out.len() < m {
        let u = rng.random_range(0..n);
        let v = rng.random_range(0..n);
        if u == v {
            continue;
        }
        let p = norm(Edge { u, v });
        if seen.insert(p) {
            out.push(p);
        }
    }
    out
}

pub fn mk_edges(edges: &[Edge]) -> Vec<Edge> {
    edges.iter().copied().map(norm).collect()
}

pub fn mask_from_spqr(dyn_ps: &PlanarSubgraph) -> Vec<bool> {
    dyn_ps.query()
}

pub fn pair_index(edges: &[Edge]) -> HashMap<Edge, usize> {
    edges
        .iter()
        .copied()
        .enumerate()
        .map(|(i, p)| (norm(p), i))
        .collect()
}

pub fn truth_addability_mask(n: usize, selected: &HashSet<usize>, edges: &[Edge]) -> Vec<bool> {
    let mut truth = Vec::with_capacity(edges.len());
    for (i, &e) in edges.iter().enumerate() {
        if selected.contains(&i) {
            truth.push(false);
            continue;
        }
        let mut cur: Vec<Edge> = selected.iter().map(|idx| edges[*idx]).collect();
        cur.push(norm(e));
        truth.push(is_planar_pairs(n, &cur));
    }
    truth
}

pub fn build_triangulated_polygon_edges(n: usize) -> Vec<Edge> {
    assert!(n >= 3);
    let mut e = Vec::new();
    for i in 0..n {
        let u = i;
        let v = (i + 1) % n;
        e.push(norm(Edge { u, v }));
    }
    for j in 2..(n - 1) {
        e.push(norm(Edge { u: 0, v: j }));
    }
    e
}

pub fn greedily_fill_to_maximal_planar(
    n: usize,
    edges_all: &[Edge],
    selected: HashSet<usize>,
) -> HashSet<usize> {
    let mut mask_init = vec![false; edges_all.len()];
    for i in &selected {
        mask_init[*i] = true;
    }
    let mut ps = PlanarSubgraph::new(n, edges_all, &mask_init);
    let mut sel = selected;
    loop {
        let mask = mask_from_spqr(&ps);
        let next = mask
            .iter()
            .enumerate()
            .find(|(i, b)| **b && !sel.contains(i));
        let Some((i, _)) = next else {
            break;
        };
        ps.set(i, true);
        sel.insert(i);
    }
    sel
}

pub fn planarize_edges_greedy(n: usize, edges: &[Edge]) -> Vec<usize> {
    let mut g: UnGraph<(), ()> = UnGraph::new_undirected();
    let nodes: Vec<_> = (0..n).map(|_| g.add_node(())).collect();
    let mut keep = Vec::new();
    for (i, &e) in edges.iter().enumerate() {
        let Edge { u, v } = norm(e);
        let a = nodes[u];
        let b = nodes[v];
        let e = g.add_edge(a, b, ());
        if is_planar(&g) {
            keep.push(i);
            continue;
        }
        g.remove_edge(e);
    }
    keep
}

pub fn cycle_edges(n: usize) -> Vec<Edge> {
    let mut e = Vec::new();
    for i in 0..n {
        let u = i;
        let v = (i + 1) % n;
        e.push(norm(Edge { u, v }));
    }
    e
}

pub fn grid_edges(w: usize, h: usize) -> Vec<Edge> {
    let mut e = Vec::new();
    let nid = |x: usize, y: usize| y * w + x;
    for y in 0..h {
        for x in 0..w {
            if x + 1 < w {
                e.push(norm(Edge {
                    u: nid(x, y),
                    v: nid(x + 1, y),
                }));
            }
            if y + 1 < h {
                e.push(norm(Edge {
                    u: nid(x, y),
                    v: nid(x, y + 1),
                }));
            }
        }
    }
    e
}

pub fn boyer_myrvold(n: usize, edges: &[Edge]) -> (bool, Vec<Edge>) {
    let planar = is_planar_pairs(n, edges);
    if planar {
        return (true, Vec::new());
    }
    let mut witness = edges.iter().copied().map(norm).collect::<Vec<_>>();
    witness.sort_by(|a, b| {
        if a.u != b.u {
            a.u.cmp(&b.u)
        } else {
            a.v.cmp(&b.v)
        }
    });
    witness.dedup();
    (false, witness)
}

fn is_planar_pairs(n: usize, pairs: &[Edge]) -> bool {
    let mut g: UnGraph<(), ()> = UnGraph::new_undirected();
    let nodes: Vec<_> = (0..n).map(|_| g.add_node(())).collect();
    for &e in pairs {
        let Edge { u, v } = norm(e);
        g.add_edge(nodes[u], nodes[v], ());
    }
    is_planar(&g)
}

pub fn compare_masks(spqr_mask: &[bool], truth_mask: &[bool], ignore: &HashSet<usize>) {
    for (i, (a, b)) in spqr_mask.iter().zip(truth_mask.iter()).enumerate() {
        if ignore.contains(&i) {
            continue;
        }
        assert_eq!(
            *a as i32, *b as i32,
            "mask mismatch at {i}: spqr={a}, truth={b}"
        );
    }
}
