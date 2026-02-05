#include "mps.hpp"

#if defined(__GNUC__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-parameter"
#endif
#include <ogdf/basic/Graph.h>
#include <ogdf/basic/SList.h>
#include <ogdf/planarity/BoyerMyrvold.h>
#if defined(__GNUC__)
#pragma GCC diagnostic pop
#endif

#include <algorithm>
#include <limits>
#include <memory>
#include <stdexcept>
#include <unordered_set>
#include <vector>

namespace graph
{

    using namespace ogdf;

    std::vector<Edge> boyer_myrvold_witness(
        std::size_t n_vertices, const std::vector<Edge> &edges)
    {
        Graph G;
        std::vector<node> nodes(n_vertices);
        for (size_t i = 0; i < n_vertices; ++i)
            nodes[i] = G.newNode();
        for (const auto &e : edges)
        {
            if (e.u >= n_vertices || e.v >= n_vertices)
            {
                throw std::out_of_range("edge endpoint out of range");
            }
            if (e.u == e.v)
            {
                continue;
            }
            G.newEdge(
                nodes[static_cast<size_t>(e.u)],
                nodes[static_cast<size_t>(e.v)]);
        }

        BoyerMyrvold bm;
        SList<KuratowskiWrapper> witnesses;
        bool planar = bm.planarEmbedDestructive(
            G, witnesses, 1,
            /*bundles=*/false,
            /*limitStructures=*/true,
            /*randomDFSTree=*/false,
            /*avoidE2Minors=*/true);

        if (planar)
            return std::vector<Edge>();
        if (witnesses.empty())
            return std::vector<Edge>();

        NodeArray<size_t> idx(G, -1);
        for (size_t i = 0; i < nodes.size(); ++i)
            idx[nodes[i]] = i;
        const KuratowskiWrapper &kw = witnesses.front();
        std::unordered_set<long long> seen;
        auto key = [](int a, int b)
        {
            if (a > b)
                std::swap(a, b);
            return (static_cast<long long>(a) << 32) | static_cast<unsigned long long>(b);
        };
        std::vector<Edge> witness_edges;
        for (edge e : kw.edgeList)
        {
            if (e == nullptr)
                continue;
            size_t u = idx[e->source()];
            size_t v = idx[e->target()];
            if (u == v)
                continue;
            if (u > v)
                std::swap(u, v);
            auto k = key(u, v);
            if (seen.count(k))
                continue;
            seen.insert(k);
            witness_edges.push_back(Edge{u, v});
        }
        std::sort(witness_edges.begin(), witness_edges.end(), [](const Edge &a, const Edge &b)
                  { return (a.u < b.u) || (a.u == b.u && a.v < b.v); });
        return witness_edges;
    }

} // namespace graph
