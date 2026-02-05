#pragma once

#include <vector>

#include "types.hpp"

namespace graph
{

    // returns an empty vector if the graph is planar
    std::vector<Edge> boyer_myrvold_witness(
        std::size_t n_vertices, const std::vector<Edge> &edges);

} // namespace graph
