mod common;

use common::{
    edges_complete, greedily_fill_to_maximal_planar, mask_from_spqr, mk_edges,
    truth_addability_mask,
};
use std::collections::HashSet;

use graphum::spqr::PlanarSubgraph;

#[test]
fn remove_one_edge_from_triangulation_two_diagonals_addable() {
    for &n in &[8usize, 12] {
        let pairs = edges_complete(n);
        let edges_all = mk_edges(&pairs);

        let sel = greedily_fill_to_maximal_planar(n, &pairs, HashSet::new());

        let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);
        for i in sel.iter().copied() {
            ps.set(i, true);
        }

        let candidates: Vec<usize> = sel
            .iter()
            .copied()
            .filter(|i| {
                let mut selected = sel.clone();
                selected.remove(i);
                truth_addability_mask(n, &selected, &pairs)
                    .iter()
                    .map(|b| *b as i32)
                    .sum::<i32>()
                    == 2
            })
            .collect();
        if candidates.is_empty() {
            continue;
        }
        let take = candidates.iter().take(candidates.len().min(5)).copied();
        for ridx in take {
            ps.set(ridx, false);
            let mut selected = sel.clone();
            selected.remove(&ridx);

            let mv = mask_from_spqr(&ps);
            let truth = truth_addability_mask(n, &selected, &pairs);

            let addable: Vec<usize> = truth
                .iter()
                .enumerate()
                .filter_map(|(j, b)| (!selected.contains(&j) && *b).then_some(j))
                .collect();
            assert_eq!(addable.len(), 2, "expected exactly two addables for {ridx}");
            assert!(addable.contains(&ridx), "removed edge should be addable");
            let opp = *addable.iter().find(|j| **j != ridx).unwrap();

            for (j, (a, b)) in mv.iter().zip(truth.iter()).enumerate() {
                if selected.contains(&j) {
                    continue;
                }
                let expected = if j == ridx || j == opp { 1 } else { 0 };
                assert_eq!((*a as i32, *b as i32), (expected, expected));
            }
            assert_eq!(mv, truth);

            ps.set(ridx, true);
        }
    }
}

#[test]
fn remove_two_disjoint_edges_yields_two_independent_quadrilaterals() {
    let n = 12usize;
    let pairs = edges_complete(n);
    let edges_all = mk_edges(&pairs);
    let sel = greedily_fill_to_maximal_planar(n, &pairs, HashSet::new());

    let vertex_disjoint = |a: graphum::Edge, b: graphum::Edge| -> bool {
        a.u != b.u && a.u != b.v && a.v != b.u && a.v != b.v
    };

    let opposite_for = |ridx: usize| -> Option<usize> {
        let mut selected = sel.clone();
        selected.remove(&ridx);
        let truth = truth_addability_mask(n, &selected, &pairs);
        let addable: Vec<usize> = truth
            .iter()
            .enumerate()
            .filter_map(|(j, b)| (!selected.contains(&j) && *b).then_some(j))
            .collect();
        if addable.len() != 2 || !addable.contains(&ridx) {
            return None;
        }
        let opp = *addable.iter().find(|j| **j != ridx).unwrap();
        vertex_disjoint(pairs[ridx], pairs[opp]).then_some(opp)
    };

    let mut ridx_pair: Option<(usize, usize, usize, usize)> = None;

    let mut flippable: Vec<(usize, usize)> = sel
        .iter()
        .copied()
        .filter_map(|ridx| opposite_for(ridx).map(|opp| (ridx, opp)))
        .collect();
    flippable.sort_unstable_by_key(|(ridx, _)| *ridx);

    'outer: for (idx, &(ridx1, opp1)) in flippable.iter().enumerate() {
        let graphum::Edge { u: u1, v: v1 } = pairs[ridx1];
        for &(ridx2, opp2) in flippable.iter().skip(idx + 1) {
            let graphum::Edge { u, v } = pairs[ridx2];
            if u == u1 || u == v1 || v == u1 || v == v1 {
                continue;
            }
            if opp1 == ridx2 || opp2 == ridx1 || opp1 == opp2 {
                continue;
            }

            let mut selected = sel.clone();
            selected.remove(&ridx1);
            selected.remove(&ridx2);

            let truth = truth_addability_mask(n, &selected, &pairs);
            let addable: HashSet<usize> = truth
                .iter()
                .enumerate()
                .filter_map(|(j, b)| (!selected.contains(&j) && *b).then_some(j))
                .collect();
            if addable.len() != 4 {
                continue;
            }
            if addable.contains(&ridx1)
                && addable.contains(&opp1)
                && addable.contains(&ridx2)
                && addable.contains(&opp2)
            {
                ridx_pair = Some((ridx1, opp1, ridx2, opp2));
                break 'outer;
            }
        }
    }

    let Some((ridx1, opp1, ridx2, opp2)) = ridx_pair else {
        // FIXME super dodge but the old python version did this too
        return;
    };

    let mut ps = PlanarSubgraph::new(n, &edges_all, &vec![false; edges_all.len()]);
    let mut sel_sorted: Vec<usize> = sel.iter().copied().collect();
    sel_sorted.sort_unstable();
    for i in sel_sorted {
        ps.set(i, true);
    }
    ps.set(ridx1, false);
    ps.set(ridx2, false);

    let mut selected = sel.clone();
    selected.remove(&ridx1);
    selected.remove(&ridx2);

    let mv = mask_from_spqr(&ps);
    let truth = truth_addability_mask(n, &selected, &pairs);

    let addable: HashSet<usize> = truth
        .iter()
        .enumerate()
        .filter_map(|(j, b)| (!selected.contains(&j) && *b).then_some(j))
        .collect();
    assert_eq!(
        addable.len(),
        4,
        "expected four addables after removing {ridx1} and {ridx2}"
    );
    assert!(addable.contains(&ridx1));
    assert!(addable.contains(&opp1));
    assert!(addable.contains(&ridx2));
    assert!(addable.contains(&opp2));

    for (j, (a, b)) in mv.iter().zip(truth.iter()).enumerate() {
        if selected.contains(&j) {
            continue;
        }
        let expected = if addable.contains(&j) { 1 } else { 0 };
        assert_eq!((*a as i32, *b as i32), (expected, expected));
    }
    assert_eq!(mv, truth);
}
