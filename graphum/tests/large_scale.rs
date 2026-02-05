mod common;

use common::{edges_random, mask_from_spqr, mk_edges};
use rand::seq::{IndexedRandom, SliceRandom};
use std::collections::HashSet;

use graphum::spqr::PlanarSubgraph;

#[test]
fn large_sparse_graph_greedy_add_remove_rounds_20k() {
    run_large_sparse_graph(20_000);
}

#[test]
fn large_sparse_graph_greedy_add_remove_rounds_50k() {
    run_large_sparse_graph(50_000);
}

fn run_large_sparse_graph(m: usize) {
    let mut rng = common::rng(0xDEADBEEF ^ m as u64);

    let n = 10_000;
    let edges = edges_random(n, m, &mut rng);
    let edges_all = mk_edges(&edges);

    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);

    let mv0 = mask_from_spqr(&ps);
    assert_eq!(mv0.len(), edges.len());
    assert!(mv0.iter().all(|b| *b));

    let mut selected: HashSet<usize> = HashSet::new();

    let rounds = 2;
    let add_cap = if m <= 20_000 { 300 } else { 400 };
    let del_cap = if m <= 20_000 { 120 } else { 160 };
    let readd_cap = if m <= 20_000 { 250 } else { 350 };

    let baseline_mask = |selected: &HashSet<usize>| {
        let mut base = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);
        for i in selected.iter().copied().collect::<Vec<_>>() {
            base.set(i, true);
        }
        mask_from_spqr(&base)
    };

    for _ in 0..rounds {
        let mut adds = 0;
        while adds < add_cap {
            let mv = mask_from_spqr(&ps);
            let legal: Vec<usize> = mv
                .iter()
                .enumerate()
                .filter_map(|(i, b)| (*b && !selected.contains(&i)).then_some(i))
                .collect();
            if legal.is_empty() {
                break;
            }
            let i = *legal.choose(&mut rng).unwrap();
            ps.set(i, true);
            selected.insert(i);
            let mv_after = mask_from_spqr(&ps);
            assert!(!mv_after[i]);
            adds += 1;
        }

        let mv_dyn = mask_from_spqr(&ps);
        let mv_base = baseline_mask(&selected);
        assert_eq!(mv_dyn, mv_base);

        let mut dels = 0;
        if !selected.is_empty() {
            let mut sel_list: Vec<_> = selected.iter().copied().collect();
            sel_list.shuffle(&mut rng);
            for ridx in sel_list {
                ps.set(ridx, false);
                selected.remove(&ridx);
                dels += 1;
                if dels >= del_cap {
                    break;
                }
            }
        }

        let mv_dyn = mask_from_spqr(&ps);
        let mv_base = baseline_mask(&selected);
        assert_eq!(mv_dyn, mv_base);

        let mut readds = 0;
        while readds < readd_cap {
            let mv = mask_from_spqr(&ps);
            let legal: Vec<usize> = mv
                .iter()
                .enumerate()
                .filter_map(|(i, b)| (*b && !selected.contains(&i)).then_some(i))
                .collect();
            if legal.is_empty() {
                break;
            }
            let i = *legal.choose(&mut rng).unwrap();
            ps.set(i, true);
            selected.insert(i);
            let mv_after = mask_from_spqr(&ps);
            assert!(!mv_after[i]);
            readds += 1;
        }

        let mv_dyn = mask_from_spqr(&ps);
        let mv_base = baseline_mask(&selected);
        assert_eq!(mv_dyn, mv_base);
    }
}
