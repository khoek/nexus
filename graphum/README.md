# graphum

Safe Rust wrappers over `ogdf-sys` for planarity testing and planar subgraph
construction.

## Features

- **Boyer--Myrvold**: Planarity test with optional Kuratowski witness.
- **SPQR subgraphs**: Planar subgraph construction via SPQR decomposition.
- **Escape hatch**: Re-exports `ogdf-sys::autogen` for direct OGDF access.

## Example

```rust
use graphum::{Edge, boyer_myrvold_witness};

let edges = vec![Edge { u: 0, v: 1 }, Edge { u: 1, v: 2 }, Edge { u: 2, v: 0 }];
let witness = boyer_myrvold_witness(3, &edges);
assert!(witness.is_none()); // triangle is planar
```

## License

AGPL-3.0-only. See `LICENSE` for details.
