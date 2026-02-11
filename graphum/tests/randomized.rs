mod common;

use common::{edges_random, mask_from_spqr, mk_edges, truth_addability_mask};
use rand::{RngExt, seq::IndexedRandom};
use std::collections::HashSet;

use graphum::spqr::PlanarSubgraph;

#[test]
fn random_sequence_matches_reset_baseline_seed_12345() {
    run_random_sequence_matches_reset_baseline(0x12345);
}

#[test]
fn random_sequence_matches_reset_baseline_seed_badc0de() {
    run_random_sequence_matches_reset_baseline(0xBADC0DE);
}

#[test]
fn random_sequence_matches_reset_baseline_seed_c0ffee() {
    run_random_sequence_matches_reset_baseline(0xC0FFEE);
}

fn run_random_sequence_matches_reset_baseline(seed: u64) {
    let mut rng = common::rng(seed);
    let n = 18;
    let m = 64;
    let steps = 120;
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
        assert_eq!(mask_dyn.len(), edges.len());
        assert_eq!(mask_dyn, mask_base);

        let legal_idx: Vec<usize> = mask_dyn
            .iter()
            .enumerate()
            .filter_map(|(i, b)| (*b && !selected.contains(&i)).then_some(i))
            .collect();
        if !legal_idx.is_empty() && (selected.is_empty() || rng.random_range(0.0..1.0) < 0.7) {
            let act = *legal_idx.choose(&mut rng).unwrap();
            ps.set(act, true);
            selected.insert(act);
        } else if !selected.is_empty() {
            let ridx = *selected
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .choose(&mut rng)
                .unwrap();
            ps.set(ridx, false);
            selected.remove(&ridx);
        }
    }
}

#[test]
fn random_sequence_matches_networkx_seed_deadbeef() {
    run_random_sequence_matches_networkx(0xDEADBEEF);
}

#[test]
fn random_sequence_matches_networkx_seed_feedface() {
    run_random_sequence_matches_networkx(0xFEEDFACE);
}

fn run_random_sequence_matches_networkx(seed: u64) {
    let mut rng = common::rng(seed);
    let n = 12;
    let m = 40;
    let steps = 64;
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
            assert_eq!(*a as i32, *b as i32, "mismatch at {}", i);
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
