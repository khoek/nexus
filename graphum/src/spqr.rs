use autocxx::moveit::Emplace;
use cxx::{CxxVector, UniquePtr};

use crate::{Edge, autogen::graph};

/// Safe Rust wrapper around the OGDF SPQR-based `graph::PlanarSubgraph`.
pub struct PlanarSubgraph {
    inner: UniquePtr<graph::PlanarSubgraph>,
}

impl PlanarSubgraph {
    pub fn new(num_verts: usize, edges_all: &[Edge], edges_added: &[bool]) -> Self {
        assert_eq!(
            edges_all.len(),
            edges_added.len(),
            "edges_all and edges_added lengths must match"
        );

        let mut edges: UniquePtr<CxxVector<graph::Edge>> = CxxVector::new();
        {
            let mut vec = edges.pin_mut();
            for &e in edges_all {
                vec.as_mut().push(e);
            }
        }

        let mut added: UniquePtr<CxxVector<u8>> = CxxVector::new();
        {
            let mut vec = added.pin_mut();
            for &b in edges_added {
                vec.as_mut().push(if b { 1 } else { 0 });
            }
        }

        let inner = unsafe {
            UniquePtr::emplace(graph::PlanarSubgraph::new(
                num_verts,
                edges.as_ref().unwrap(),
                added.as_ref().unwrap(),
            ))
        };

        Self { inner }
    }

    pub fn set(&mut self, edge_id: usize, present: bool) {
        let mut inner = self.inner.pin_mut();
        unsafe { inner.as_mut().set(edge_id, present) };
    }

    pub fn query(&self) -> Vec<bool> {
        let inner = self.inner.as_ref().unwrap();

        let mask: UniquePtr<CxxVector<u8>> = unsafe { inner.query() };

        mask.as_ref()
            .unwrap()
            .as_slice()
            .iter()
            .map(|&b| b != 0)
            .collect()
    }

    /// Expose the raw C++ pointer if you ever need to call other C++ APIs.
    pub fn as_raw(&self) -> &UniquePtr<graph::PlanarSubgraph> {
        &self.inner
    }
}
