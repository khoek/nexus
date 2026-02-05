use crate::{Edge, autogen::graph};
use cxx::{CxxVector, UniquePtr};

/// Run the Boyer–Myrvold planarity test.
///
/// Returns:
///   * `None`  – the graph is planar.
///   * `Some(edges)` – a Kuratowski witness subgraph when the graph is non-planar.
///     The edges are returned as a list of (u, v) with 0-based vertex indices.
///     If OGDF returns an empty vector for some degenerate non-planar case,
///     this is surfaced as `Some(Vec::new())`.
pub fn boyer_myrvold_witness(num_verts: usize, edges: &[Edge]) -> Option<Vec<Edge>> {
    let mut edge_buf: UniquePtr<CxxVector<Edge>> = CxxVector::new();
    {
        let mut vec = edge_buf.pin_mut();
        for &e in edges {
            vec.as_mut().push(e);
        }
    }

    let witness: UniquePtr<CxxVector<graph::Edge>> =
        unsafe { graph::boyer_myrvold_witness(num_verts, edge_buf.as_ref().unwrap()) };

    let out: Vec<Edge> = witness.iter().map(|e| Edge { u: e.u, v: e.v }).collect();
    (!out.is_empty()).then_some(out)
}
