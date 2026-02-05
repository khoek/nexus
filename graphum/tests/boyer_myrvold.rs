mod common;

use common::{boyer_myrvold, edge, edges_complete};

fn complete_pairs(n: usize) -> Vec<graphum::Edge> {
    edges_complete(n)
}

fn cycle_pairs(n: usize) -> Vec<graphum::Edge> {
    (0..n).map(|i| edge(i, (i + 1) % n)).collect()
}

fn k33_pairs() -> Vec<graphum::Edge> {
    let left = [0, 1, 2];
    let right = [3, 4, 5];
    let mut pairs = Vec::new();
    for &u in &left {
        for &v in &right {
            pairs.push(edge(u, v));
        }
    }
    pairs
}

#[test]
fn planar_cycle_is_planar_with_no_witness() {
    let n = 8usize;
    let pairs = cycle_pairs(n);
    let (planar, witness) = boyer_myrvold(n, &pairs);
    assert!(planar);
    assert!(witness.is_empty());
}

#[test]
fn k5_is_non_planar_and_returns_witness_edges() {
    let n = 5usize;
    let pairs = complete_pairs(n);
    let (planar, witness) = boyer_myrvold(n, &pairs);
    assert!(!planar);
    assert!(!witness.is_empty());
    let pairset: std::collections::HashSet<_> = pairs.iter().copied().collect();
    for e in witness {
        assert!(pairset.contains(&e));
    }
}

#[test]
fn k33_is_non_planar_and_returns_witness_edges() {
    let n = 6usize;
    let pairs = k33_pairs();
    let (planar, witness) = boyer_myrvold(n, &pairs);
    assert!(!planar);
    assert!(!witness.is_empty());
    let pairset: std::collections::HashSet<_> = pairs.iter().copied().collect();
    for e in witness {
        assert!(pairset.contains(&e));
    }
}
