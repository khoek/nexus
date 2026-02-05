mod common;

use common::{
    build_triangulated_polygon_edges, edges_complete, mask_from_spqr, mk_edges, pair_index,
};
use rand::seq::{IndexedRandom, SliceRandom};

use graphum::spqr::PlanarSubgraph;

#[test]
fn reordering_candidate_list_does_not_change_masks() {
    let mut rng = common::rng(0xD00D);

    let n = 10;
    let pairs = edges_complete(n);
    let idx = pair_index(&pairs);

    let mut pairs_perm = pairs.clone();
    pairs_perm.shuffle(&mut rng);
    let idx_perm = pair_index(&pairs_perm);

    let ea1 = mk_edges(&pairs);
    let ea2 = mk_edges(&pairs_perm);
    let mut ps1 = PlanarSubgraph::new(n, &ea1, &vec![false; ea1.len()]);
    let mut ps2 = PlanarSubgraph::new(n, &ea2, &vec![false; ea2.len()]);

    let base_pairs = build_triangulated_polygon_edges(n);
    let sel_idx: Vec<usize> = base_pairs.iter().map(|edge| idx[edge]).collect();
    for &i in &sel_idx {
        ps1.set(i, true);
        ps2.set(idx_perm[&pairs[i]], true);
    }

    for _ in 0..20 {
        let m1 = mask_from_spqr(&ps1);
        let m2p = mask_from_spqr(&ps2);
        let inv: std::collections::HashMap<usize, usize> =
            idx_perm.iter().map(|(p, j)| (*j, idx[p])).collect();
        let mut m2 = vec![false; m2p.len()];
        for (j, b) in m2p.iter().enumerate() {
            m2[inv[&j]] = *b;
        }
        assert_eq!(m1, m2);

        let legal: Vec<usize> = m1
            .iter()
            .enumerate()
            .filter_map(|(i, b)| (*b).then_some(i))
            .collect();
        if legal.is_empty() {
            break;
        }
        let i = *legal.choose(&mut rng).unwrap();
        ps1.set(i, true);
        ps2.set(idx_perm[&pairs[i]], true);
    }
}
