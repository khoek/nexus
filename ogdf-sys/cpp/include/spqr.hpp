#pragma once

#include <cstdint>
#include <memory>
#include <vector>

#include "types.hpp"

namespace graph
{

  class PlanarSubgraph
  {
  public:
    using Mask = std::uint8_t;

    PlanarSubgraph(std::size_t num_verts,
                   const std::vector<Edge> &edges_all,
                   const std::vector<std::uint8_t> &edges_added_init);

    PlanarSubgraph(const PlanarSubgraph &) = delete;
    PlanarSubgraph &operator=(const PlanarSubgraph &) = delete;
    PlanarSubgraph(PlanarSubgraph &&) noexcept;
    PlanarSubgraph &operator=(PlanarSubgraph &&) noexcept;
    ~PlanarSubgraph();

    void set(std::size_t edge_id, bool present);
    std::vector<std::uint8_t> query() const;

  private:
    struct Impl;
    std::unique_ptr<Impl> impl;
  };

} // namespace graph
