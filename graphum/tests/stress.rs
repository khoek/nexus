mod common;

use common::{edges_random, mask_from_spqr, mk_edges};
use rand::{RngExt, seq::IndexedRandom};
use std::collections::HashSet;

use graphum::spqr::PlanarSubgraph;

#[test]
fn rebuild_baseline_matches_dynamic_on_large_sequence() {
    let mut rng = common::rng(0xABAD1DEA);

    let n = 64;
    let m = 256;
    let steps = 200;
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
        if !legal_idx.is_empty() && (selected.is_empty() || rng.random_range(0.0..1.0) < 0.65) {
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
