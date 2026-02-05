mod common;

use common::{edges_random, mask_from_spqr, mk_edges, rng};
use graphum::spqr::PlanarSubgraph;

#[test]
fn idempotent_add_remove_noops_seed_dead() {
    run_idempotent_add_remove(0xDEAD);
}

#[test]
fn idempotent_add_remove_noops_seed_beef() {
    run_idempotent_add_remove(0xBEEF);
}

fn run_idempotent_add_remove(seed: u64) {
    let mut rng = rng(seed);
    let n = 14;
    let m = 60;
    let edges = edges_random(n, m, &mut rng);
    let edges_all = mk_edges(&edges);
    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);

    let m0 = mask_from_spqr(&ps);

    let legal: Vec<usize> = m0
        .iter()
        .enumerate()
        .filter_map(|(i, b)| (*b).then_some(i))
        .collect();
    let take: Vec<usize> = legal.into_iter().take(8).collect();

    for &i in &take {
        ps.set(i, true);
        ps.set(i, true);
    }
    let m1 = mask_from_spqr(&ps);
    for &i in &take {
        assert_eq!(
            m1[i] as i32, 0,
            "index {} should be non-addable after add",
            i
        );
    }

    for &i in take.iter().take(3) {
        ps.set(i, false);
        ps.set(i, false);
    }
    let m2 = mask_from_spqr(&ps);
    assert_eq!(m2.len(), m1.len());
}
