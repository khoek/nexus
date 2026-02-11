mod common;

use common::{edges_random, mask_from_spqr, mk_edges, truth_addability_mask};
use rand::{RngExt, seq::IndexedRandom};
use std::collections::HashSet;

use graphum::spqr::PlanarSubgraph;

#[test]
fn dynamic_vs_reset_extensive() {
    for seed in [0x111u64, 0x222, 0x333, 0x444, 0x555] {
        run_dynamic_vs_reset(seed);
    }
}

fn run_dynamic_vs_reset(seed: u64) {
    let mut rng = common::rng(seed);
    let n = 20;
    let m = 80;
    let steps = 160;
    let edges = edges_random(n, m, &mut rng);
    let edges_all = mk_edges(&edges);
    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);
    let mut selected: HashSet<usize> = HashSet::new();

    for _ in 0..steps {
        let mut base = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);
        for i in selected.iter().copied().collect::<Vec<_>>() {
            base.set(i, true);
        }
        let mask_dyn = mask_from_spqr(&ps);
        let mask_base = mask_from_spqr(&base);
        assert_eq!(mask_dyn, mask_base);

        let legal_idx: Vec<usize> = mask_dyn
            .iter()
            .enumerate()
            .filter_map(|(i, b)| (*b && !selected.contains(&i)).then_some(i))
            .collect();
        if !legal_idx.is_empty() && (selected.is_empty() || rng.random_range(0.0..1.0) < 0.7) {
            let i = *legal_idx.choose(&mut rng).unwrap();
            ps.set(i, true);
            selected.insert(i);
        } else if !selected.is_empty() {
            let i = *selected
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .choose(&mut rng)
                .unwrap();
            ps.set(i, false);
            selected.remove(&i);
        }
    }
}

#[test]
fn dynamic_vs_networkx_extensive() {
    for seed in [0xAAA0u64, 0xAAA1, 0xAAA2, 0xAAA3] {
        run_dynamic_vs_networkx(seed);
    }
}

fn run_dynamic_vs_networkx(seed: u64) {
    let mut rng = common::rng(seed);
    let n = 12;
    let m = 50;
    let steps = 100;
    let edges = edges_random(n, m, &mut rng);
    let edges_all = mk_edges(&edges);
    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);
    let mut selected: HashSet<usize> = HashSet::new();

    for _ in 0..steps {
        let mask_dyn = mask_from_spqr(&ps);
        let truth = truth_addability_mask(n, &selected, &edges);
        for (i, (a, b)) in mask_dyn.iter().zip(truth.iter()).enumerate() {
            if selected.contains(&i) {
                continue;
            }
            assert_eq!(*a as i32, *b as i32, "mismatch at {i}");
        }

        let legal_idx: Vec<usize> = mask_dyn
            .iter()
            .enumerate()
            .filter_map(|(i, b)| (*b && !selected.contains(&i)).then_some(i))
            .collect();
        if !legal_idx.is_empty() && (selected.is_empty() || rng.random_range(0.0..1.0) < 0.6) {
            let i = *legal_idx.choose(&mut rng).unwrap();
            ps.set(i, true);
            selected.insert(i);
        } else if !selected.is_empty() {
            let i = *selected
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .choose(&mut rng)
                .unwrap();
            ps.set(i, false);
            selected.remove(&i);
        }
    }
}
