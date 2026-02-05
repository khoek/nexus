#[cxx::bridge(namespace = "graph")]
mod pod {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct Edge {
        pub u: usize,
        pub v: usize,
    }

    impl CxxVector<Edge> {}
}

pub use pod::Edge;

#[allow(clippy::all, unsafe_op_in_unsafe_fn)]
pub mod autogen {
    use autocxx::prelude::*;

    include_cpp! {
        #include "types.hpp"
        #include "spqr.hpp"
        #include "mps.hpp"

        extern_cpp_type!("graph::Edge", crate::pod::Edge)

        generate!("graph::boyer_myrvold_witness")
        generate!("graph::PlanarSubgraph")
    }

    pub use ffi::graph;
}
