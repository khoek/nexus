#include "spqr.hpp"

#if defined(__GNUC__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-parameter"
#endif
#include <ogdf/basic/CombinatorialEmbedding.h>
#include <ogdf/basic/Graph.h>
#include <ogdf/basic/Graph_d.h>
#include <ogdf/basic/simple_graph_alg.h>
#include <ogdf/decomposition/BCTree.h>
#include <ogdf/decomposition/StaticPlanarSPQRTree.h>
#if defined(__GNUC__)
#pragma GCC diagnostic pop
#endif

#include <algorithm>
#include <array>
#include <cstddef>
#include <cstdint>
#include <memory>
#include <mutex>
#include <optional>
#include <stdexcept>
#include <unordered_map>
#include <vector>

namespace graph
{

    struct BCTreeX : ogdf::BCTree
    {
        using ogdf::BCTree::BCTree;
        using ogdf::BCTree::findNCA;
        using ogdf::BCTree::parent;
    };

    using Mask = PlanarSubgraph::Mask;
    struct BlockInfo;

    struct PlanarSubgraph::Impl
    {
        Impl(std::size_t num_verts,
             const std::vector<Edge> &edges_all,
             const std::vector<std::uint8_t> &edges_added_init);

        Impl(const Impl &) = delete;
        Impl &operator=(const Impl &) = delete;
        Impl(Impl &&) = delete;
        Impl &operator=(Impl &&) = delete;
        ~Impl();

        void set(std::size_t edge_id, bool present);
        std::vector<std::uint8_t> query() const;

    private:
        void recalculate_addable(const std::vector<std::uint8_t> &added_snapshot);
        bool can_add_along_bc(const BCTreeX &bc, ogdf::node uG, ogdf::node vG);
        BlockInfo &get_or_build_block(const BCTreeX &bc, ogdf::node vB);
        void reset_blocks(const ogdf::Graph &bc_tree);

        const std::size_t num_verts;
        const std::vector<Edge> edges_all;

        std::vector<std::uint8_t> edges_added;
        mutable std::shared_ptr<const std::vector<std::uint8_t>> addable_snap;
        mutable std::mutex mu;

        ogdf::Graph G;
        std::vector<ogdf::node> v_g;
        std::vector<ogdf::edge> cur_edges;
        std::unique_ptr<ogdf::NodeArray<BlockInfo *>> blocks;
        std::vector<std::unique_ptr<BlockInfo>> owned_blocks;
    };

    inline std::pair<ogdf::face, ogdf::face> faces_of(const ogdf::ConstCombinatorialEmbedding &CE, ogdf::adjEntry a)
    {
        return {CE.leftFace(a), CE.rightFace(a)};
    }

    inline std::pair<ogdf::face, ogdf::face> faces_of(const ogdf::ConstCombinatorialEmbedding &CE, ogdf::edge e)
    {
        return faces_of(CE, e->adjSource());
    }

    inline bool contains_face_id(const std::vector<uint32_t> &v, uint32_t id)
    {
        return std::binary_search(v.begin(), v.end(), id);
    }

    inline ogdf::edge skel_edge_at(const ogdf::StaticPlanarSPQRTree &spqr, ogdf::edge e_t, ogdf::node v_t)
    {
        return (e_t->source() == v_t) ? spqr.skeletonEdgeSrc(e_t) : spqr.skeletonEdgeTgt(e_t);
    }

    struct BlockInfo
    {
        ogdf::Graph block;
        ogdf::NodeArray<ogdf::node> h_to_b;

        std::unique_ptr<ogdf::StaticPlanarSPQRTree> spqr;
        const ogdf::Graph *tree = nullptr;

        ogdf::NodeArray<ogdf::node> parent_t;
        ogdf::NodeArray<ogdf::edge> parent_edge_t;
        ogdf::NodeArray<int> depth_t;
        std::vector<ogdf::NodeArray<ogdf::node>> up;
        ogdf::NodeArray<int> tin, tout;

        ogdf::NodeArray<ogdf::node> repr_t;

        ogdf::NodeArray<std::optional<ogdf::ConstCombinatorialEmbedding>> emb_r;

        struct FaceIndex
        {
            std::unordered_map<size_t, uint32_t> face_id;
            std::unordered_map<size_t, std::vector<uint32_t>> v_faces;
            std::unordered_map<size_t, std::array<uint32_t, 2>> e_faces;
            uint32_t next_id = 0;
        };
        ogdf::NodeArray<FaceIndex *> r_face_index;
        std::vector<std::unique_ptr<FaceIndex>> r_face_owned;

        struct PairKey
        {
            ogdf::node u{nullptr}, v{nullptr};
        };
        struct PairKeyHash
        {
            size_t operator()(const PairKey &k) const noexcept
            {
                size_t a = (size_t)(uintptr_t)k.u;
                size_t b = (size_t)(uintptr_t)k.v;
                if (a > b)
                {
                    std::swap(a, b);
                }
                return a * 1315423911u ^ (b + 0x9e3779b97f4a7c15ull + (a << 6) + (a >> 2));
            }
        };
        struct PairKeyEq
        {
            bool operator()(const PairKey &x, const PairKey &y) const noexcept
            {
                size_t ax = (size_t)(uintptr_t)x.u, bx = (size_t)(uintptr_t)x.v;
                size_t ay = (size_t)(uintptr_t)y.u, by = (size_t)(uintptr_t)y.v;
                if (ax > bx)
                {
                    std::swap(ax, bx);
                }
                if (ay > by)
                {
                    std::swap(ay, by);
                }
                return ax == ay && bx == by;
            }
        };
        mutable std::unordered_map<PairKey, bool, PairKeyHash, PairKeyEq> memo;

        BlockInfo(const ogdf::BCTree &bc, ogdf::node vB)
        {
            const ogdf::Graph &H = bc.auxiliaryGraph();

            h_to_b.init(H, nullptr);
            for (ogdf::edge eH : bc.hEdges(vB))
            {
                ogdf::node xH = eH->source(), yH = eH->target();
                if (!h_to_b[xH])
                {
                    h_to_b[xH] = block.newNode();
                }
                if (!h_to_b[yH])
                {
                    h_to_b[yH] = block.newNode();
                }
                block.newEdge(h_to_b[xH], h_to_b[yH]);
            }

            const int nv = block.numberOfNodes();
            const int ne = block.numberOfEdges();
            if (nv < 2 || (nv == 2 && ne < 3))
            {
                static ogdf::Graph k_empty_graph;
                spqr = nullptr;
                tree = &k_empty_graph;
                repr_t.init(block, nullptr);
                emb_r.init(k_empty_graph);
                return;
            }

            spqr = std::make_unique<ogdf::StaticPlanarSPQRTree>(block, false);
            tree = &spqr->tree();

            repr_t.init(block, nullptr);
            for (ogdf::node vT = tree->firstNode(); vT; vT = vT->succ())
            {
                const ogdf::Skeleton &Sk = spqr->skeleton(vT);
                for (ogdf::node xM = Sk.getGraph().firstNode(); xM; xM = xM->succ())
                {
                    ogdf::node orig = Sk.original(xM);
                    if (!repr_t[orig])
                    {
                        repr_t[orig] = vT;
                    }
                }
            }
            emb_r.init(*tree);
            r_face_index.init(*tree, nullptr);
            build_lca();
        }

        ogdf::ConstCombinatorialEmbedding &embedding_r(ogdf::node vT)
        {
            auto &slot = emb_r[vT];
            if (!slot)
            {
                const ogdf::Skeleton &S = spqr->skeleton(vT);
                slot = ogdf::ConstCombinatorialEmbedding(S.getGraph());
            }
            return *slot;
        }

        uint32_t fi_get_or_assign(ogdf::node vT, ogdf::face f)
        {
            auto &FI = *r_face_index[vT];
            size_t key = (size_t)(uintptr_t)f;
            auto it = FI.face_id.find(key);
            if (it != FI.face_id.end())
            {
                return it->second;
            }
            uint32_t id = FI.next_id++;
            FI.face_id.emplace(key, id);
            return id;
        }

        void ensure_face_index(ogdf::node vT)
        {
            if (spqr->typeOf(vT) != ogdf::SPQRTree::NodeType::RNode)
            {
                return;
            }
            if (r_face_index[vT])
            {
                return;
            }
            r_face_owned.emplace_back(std::make_unique<BlockInfo::FaceIndex>());
            r_face_index[vT] = r_face_owned.back().get();
        }

        const std::vector<uint32_t> &vertex_face_ids(ogdf::node vT, ogdf::node bB)
        {
            ensure_face_index(vT);
            auto &FI = *r_face_index[vT];
            size_t key = (size_t)(uintptr_t)bB;
            auto it = FI.v_faces.find(key);
            if (it != FI.v_faces.end())
            {
                return it->second;
            }

            const ogdf::Skeleton &S = spqr->skeleton(vT);
            auto &CE = embedding_r(vT);
            std::vector<uint32_t> vec;
            for (ogdf::node xM = S.getGraph().firstNode(); xM; xM = xM->succ())
            {
                if (S.original(xM) != bB)
                {
                    continue;
                }
                for (ogdf::adjEntry a = xM->firstAdj(); a; a = a->succ())
                {
                    auto ff = faces_of(CE, a);
                    vec.push_back(fi_get_or_assign(vT, ff.first));
                    vec.push_back(fi_get_or_assign(vT, ff.second));
                }
            }
            std::sort(vec.begin(), vec.end());
            vec.erase(std::unique(vec.begin(), vec.end()), vec.end());
            auto [pos, _] = FI.v_faces.emplace(key, std::move(vec));
            return pos->second;
        }

        std::array<uint32_t, 2> edge_face_ids(ogdf::node vT, ogdf::edge eT)
        {
            ensure_face_index(vT);
            auto &FI = *r_face_index[vT];
            size_t key = (size_t)(uintptr_t)eT;
            auto it = FI.e_faces.find(key);
            if (it != FI.e_faces.end())
            {
                return it->second;
            }

            auto &CE = embedding_r(vT);
            ogdf::edge e_sk = skel_edge_at(*spqr, eT, vT);
            auto ff = faces_of(CE, e_sk);
            std::array<uint32_t, 2> ids = {fi_get_or_assign(vT, ff.first), fi_get_or_assign(vT, ff.second)};
            FI.e_faces.emplace(key, ids);
            return ids;
        }

        Mask endpoint_mask_at(ogdf::node v_t, ogdf::edge up_edge, ogdf::node uB);
        std::pair<ogdf::edge, Mask> walk_side_to_lca(ogdf::node start_t, ogdf::node lca_t, ogdf::node endpoint_b);
        ogdf::node walk_up_containing(ogdf::node start, ogdf::node stop, ogdf::node bB) const;
        ogdf::node walk_down_containing(ogdf::node from, ogdf::node toward, ogdf::node bB) const;
        std::pair<ogdf::node, ogdf::node> compute_boundaries(ogdf::node aT, ogdf::node bT, ogdf::node w,
                                                             ogdf::node aB, ogdf::node bB) const;
        bool edge_contains_b(ogdf::node parent, ogdf::node child, ogdf::node bB) const;
        ogdf::node kth_ancestor(ogdf::node v, int k) const;
        bool is_ancestor(ogdf::node a, ogdf::node b) const;
        bool is_on_path_up_to_lca(ogdf::node x, ogdf::node start, ogdf::node lca) const;
        ogdf::node lca(ogdf::node a, ogdf::node b) const;
        bool cofacial_at_node(ogdf::node vT, ogdf::node aB, ogdf::node bB);
        bool block_linkable(ogdf::node hA, ogdf::node hB);

        BlockInfo(const BlockInfo &) = delete;
        BlockInfo &operator=(const BlockInfo &) = delete;
        BlockInfo(BlockInfo &&) = default;
        BlockInfo &operator=(BlockInfo &&) = default;

    private:
        void build_lca();
    };

    bool BlockInfo::edge_contains_b(ogdf::node parent, ogdf::node child, ogdf::node bB) const
    {
        ogdf::edge e_up = parent_edge_t[child];
        if (!e_up)
        {
            return false;
        }
        const ogdf::Skeleton &sk = spqr->skeleton(parent);
        ogdf::edge e_sk = skel_edge_at(*spqr, e_up, parent);
        return sk.original(e_sk->source()) == bB || sk.original(e_sk->target()) == bB;
    }

    ogdf::node BlockInfo::kth_ancestor(ogdf::node v, int k) const
    {
        if (!v || k < 0)
        {
            return nullptr;
        }
        for (int i = 0; v && k; ++i)
        {
            if (k & 1)
            {
                v = up[i][v];
            }
            k >>= 1;
        }
        return v;
    }

    bool BlockInfo::is_ancestor(ogdf::node a, ogdf::node b) const
    {
        return tin[a] <= tin[b] && tout[b] <= tout[a];
    }

    bool BlockInfo::is_on_path_up_to_lca(ogdf::node x, ogdf::node start, ogdf::node lca) const
    {
        return is_ancestor(x, start) && is_ancestor(lca, x);
    }

    void BlockInfo::build_lca()
    {
        const ogdf::Graph &T = *tree;
        parent_t.init(T, nullptr);
        parent_edge_t.init(T, nullptr);
        depth_t.init(T, 0);

        ogdf::node root = spqr->rootNode();

        std::vector<ogdf::node> st{root};
        st.reserve(T.numberOfNodes());
        parent_t[root] = nullptr;
        parent_edge_t[root] = nullptr;

        while (!st.empty())
        {
            ogdf::node v = st.back();
            st.pop_back();
            for (ogdf::adjEntry a = v->firstAdj(); a; a = a->succ())
            {
                ogdf::edge e = a->theEdge();
                ogdf::node w = a->twinNode();
                if (w == parent_t[v])
                {
                    continue;
                }
                parent_t[w] = v;
                parent_edge_t[w] = e;
                depth_t[w] = depth_t[v] + 1;
                st.push_back(w);
            }
        }

        const int n = T.numberOfNodes();
        int LOG = 1;
        while ((1 << LOG) <= n)
        {
            ++LOG;
        }
        up.clear();
        up.reserve(LOG);
        up.emplace_back(T, nullptr);
        for (ogdf::node v = T.firstNode(); v; v = v->succ())
        {
            up[0][v] = parent_t[v];
        }
        for (int k = 1; k < LOG; ++k)
        {
            up.emplace_back(T, nullptr);
            for (ogdf::node v = T.firstNode(); v; v = v->succ())
            {
                ogdf::node mid = up[k - 1][v];
                up[k][v] = mid ? up[k - 1][mid] : nullptr;
            }
        }

        tin.init(T, 0);
        tout.init(T, 0);
        int timer = 0;
        std::vector<std::pair<ogdf::node, ogdf::adjEntry>> dfsSt;
        dfsSt.reserve(T.numberOfNodes());
        tin[root] = ++timer;
        dfsSt.emplace_back(root, root->firstAdj());
        while (!dfsSt.empty())
        {
            auto &fr = dfsSt.back();
            ogdf::node v = fr.first;
            ogdf::adjEntry &it = fr.second;
            while (it && parent_t[it->twinNode()] != v)
            {
                it = it->succ();
            }
            if (it)
            {
                ogdf::node w = it->twinNode();
                it = it->succ();
                tin[w] = ++timer;
                dfsSt.emplace_back(w, w->firstAdj());
            }
            else
            {
                tout[v] = timer;
                dfsSt.pop_back();
            }
        }
    }

    ogdf::node BlockInfo::lca(ogdf::node a, ogdf::node b) const
    {
        if (a == b)
        {
            return a;
        }
        int da = depth_t[a], db = depth_t[b];
        if (da < db)
        {
            std::swap(a, b), std::swap(da, db);
        }
        int diff = da - db;
        for (int k = static_cast<int>(up.size()) - 1; k >= 0; --k)
        {
            if ((diff >> k) & 1)
            {
                a = up[k][a];
            }
        }
        if (a == b)
        {
            return a;
        }
        for (int k = static_cast<int>(up.size()) - 1; k >= 0; --k)
        {
            if (up[k][a] != up[k][b])
            {
                a = up[k][a];
                b = up[k][b];
            }
        }
        return parent_t[a];
    }

    Mask BlockInfo::endpoint_mask_at(ogdf::node v_t, ogdf::edge up_edge, ogdf::node uB)
    {
        if (!up_edge)
        {
            return 0;
        }
        if (spqr->typeOf(v_t) != ogdf::SPQRTree::NodeType::RNode)
        {
            return 3;
        }
        auto ef = edge_face_ids(v_t, up_edge);
        const auto &vf = vertex_face_ids(v_t, uB);
        Mask m = 0;
        if (contains_face_id(vf, ef[0]))
        {
            m |= 1;
        }
        if (contains_face_id(vf, ef[1]))
        {
            m |= 2;
        }
        return m;
    }

    std::pair<ogdf::edge, Mask> BlockInfo::walk_side_to_lca(ogdf::node start_t, ogdf::node lca_t,
                                                            ogdf::node endpoint_b)
    {
        if (start_t == lca_t)
        {
            return {nullptr, 0};
        }

        ogdf::edge e_up = parent_edge_t[start_t];
        ogdf::node parent_at_boundary = parent_t[start_t];

        Mask seed_mask = endpoint_mask_at(parent_at_boundary, e_up, endpoint_b);
        if (seed_mask == 0 && spqr->typeOf(parent_at_boundary) == ogdf::SPQRTree::NodeType::RNode)
        {
            seed_mask = 3;
        }
        if (!seed_mask)
        {
            return {e_up, 0};
        }

        int steps_to_lca = depth_t[parent_at_boundary] - depth_t[lca_t];
        if (steps_to_lca == 0)
        {
            return {e_up, seed_mask};
        }

        Mask mask = seed_mask;
        ogdf::node child = start_t;
        for (int s = 0; s < steps_to_lca; ++s)
        {
            ogdf::node cur = parent_t[child];
            if (cur == lca_t)
            {
                break;
            }

            if (spqr->typeOf(cur) == ogdf::SPQRTree::NodeType::RNode && mask)
            {
                mask = 3;
            }

            if (spqr->typeOf(cur) == ogdf::SPQRTree::NodeType::RNode)
            {
                auto fin = edge_face_ids(cur, parent_edge_t[child]);
                auto fout = edge_face_ids(cur, parent_edge_t[cur]);
                Mask next_mask = 0;
                if (mask & 1)
                {
                    if (fin[0] == fout[0])
                    {
                        next_mask |= 1;
                    }
                    if (fin[0] == fout[1])
                    {
                        next_mask |= 2;
                    }
                }
                if (mask & 2)
                {
                    if (fin[1] == fout[0])
                    {
                        next_mask |= 1;
                    }
                    if (fin[1] == fout[1])
                    {
                        next_mask |= 2;
                    }
                }
                mask = next_mask;
            }
            if (!mask)
            {
                break;
            }
            child = cur;
        }
        if (!mask)
        {
            return {e_up, 0};
        }

        ogdf::node child_below_lca = kth_ancestor(parent_at_boundary, steps_to_lca - 1);
        ogdf::edge incoming_at_lca = child_below_lca ? parent_edge_t[child_below_lca] : e_up;
        return {incoming_at_lca, mask};
    }

    ogdf::node BlockInfo::walk_up_containing(ogdf::node start, ogdf::node stop, ogdf::node bB) const
    {
        ogdf::node repr = repr_t[bB];
        ogdf::node cur = start;
        for (int k = static_cast<int>(up.size()) - 1; k >= 0; --k)
        {
            ogdf::node cand = up[k][cur];
            if (!cand || depth_t[cand] < depth_t[stop])
            {
                continue;
            }
            if (repr == cand)
            {
                cur = cand;
            }
            else if (is_ancestor(cand, repr))
            {
                const int dist = depth_t[repr] - depth_t[cand];
                ogdf::node child = kth_ancestor(repr, dist - 1);
                if (child && edge_contains_b(cand, child, bB))
                {
                    cur = cand;
                }
            }
        }
        return cur;
    }

    ogdf::node BlockInfo::walk_down_containing(ogdf::node from, ogdf::node toward, ogdf::node bB) const
    {
        if (!from || !toward)
        {
            return from;
        }
        int dist = depth_t[toward] - depth_t[from];
        if (dist <= 0)
        {
            return from;
        }

        for (int k = static_cast<int>(up.size()) - 1; k >= 0; --k)
        {
            int step = 1 << k;
            if (step > dist)
            {
                continue;
            }

            ogdf::node child = kth_ancestor(toward, dist - step);
            ogdf::node parent = parent_t[child];
            if (edge_contains_b(parent, child, bB))
            {
                dist -= step;
            }
        }
        return kth_ancestor(toward, dist);
    }

    std::pair<ogdf::node, ogdf::node> BlockInfo::compute_boundaries(ogdf::node aT, ogdf::node bT, ogdf::node w,
                                                                    ogdf::node aB, ogdf::node bB) const
    {
        ogdf::node aBoundary = walk_up_containing(aT, w, aB);
        if (aBoundary == w)
        {
            aBoundary = walk_down_containing(w, bT, aB);
        }

        ogdf::node bBoundary = walk_up_containing(bT, w, bB);
        if (bBoundary == w)
        {
            bBoundary = walk_down_containing(w, aT, bB);
        }

        return {aBoundary, bBoundary};
    }

    inline bool share_faces(std::pair<ogdf::face, ogdf::face> A, std::pair<ogdf::face, ogdf::face> B)
    {
        return A.first == B.first || A.first == B.second || A.second == B.first || A.second == B.second;
    }

    bool BlockInfo::cofacial_at_node(ogdf::node vT, ogdf::node aB, ogdf::node bB)
    {
        if (spqr->typeOf(vT) != ogdf::SPQRTree::NodeType::RNode)
        {
            return true;
        }
        const auto &A = vertex_face_ids(vT, aB);
        const auto &B = vertex_face_ids(vT, bB);
        size_t i = 0, j = 0;
        while (i < A.size() && j < B.size())
        {
            if (A[i] < B[j])
            {
                ++i;
            }
            else if (A[i] > B[j])
            {
                ++j;
            }
            else
            {
                return true;
            }
        }
        return false;
    }

    bool BlockInfo::block_linkable(ogdf::node hA, ogdf::node hB)
    {
        if (!spqr)
        {
            return true;
        }

        ogdf::node aB = h_to_b[hA];
        ogdf::node bB = h_to_b[hB];
        if (!aB || !bB)
        {
            return false;
        }
        if (aB == bB)
        {
            return true;
        }

        ogdf::node aT = repr_t[aB];
        ogdf::node bT = repr_t[bB];
        if (!aT || !bT)
        {
            return false;
        }

        BlockInfo::PairKey key{aB, bB};
        if (auto it = memo.find(key); it != memo.end())
        {
            return it->second;
        }
        auto memo_return = [&](bool v)
        {
            memo.emplace(key, v);
            return v;
        };

        if (aT == bT)
        {
            return memo_return(cofacial_at_node(aT, aB, bB));
        }

        ogdf::node w = lca(aT, bT);

        auto [a_boundary, b_boundary] = compute_boundaries(aT, bT, w, aB, bB);
        if (a_boundary != b_boundary)
        {
            auto crossed = [&](ogdf::node path_start, ogdf::node first_b, ogdf::node second_b)
            {
                if (!is_on_path_up_to_lca(second_b, path_start, w))
                {
                    return false;
                }
                bool first_on = is_on_path_up_to_lca(first_b, path_start, w);
                return !first_on || (depth_t[second_b] > depth_t[first_b]);
            };
            if (crossed(aT, a_boundary, b_boundary) || crossed(bT, b_boundary, a_boundary))
            {
                return memo_return(true);
            }
        }

        ogdf::node start_left = a_boundary;
        ogdf::node start_right = b_boundary;
        w = lca(start_left, start_right);

        auto [left_into_w, left_mask] = walk_side_to_lca(start_left, w, aB);
        auto [right_into_w, right_mask] = walk_side_to_lca(start_right, w, bB);

        bool left_seed_ok = endpoint_mask_at(start_left, parent_edge_t[start_left], aB) != 0;
        bool right_seed_ok = endpoint_mask_at(start_right, parent_edge_t[start_right], bB) != 0;

        bool left_ok = (start_left == w) || (left_seed_ok && left_mask != 0);
        bool right_ok = (start_right == w) || (right_seed_ok && right_mask != 0);
        if (!left_ok || !right_ok)
        {
            return memo_return(false);
        }

        if (spqr->typeOf(w) != ogdf::SPQRTree::NodeType::RNode)
        {
            return memo_return(true);
        }

        if (left_into_w && right_into_w)
        {
            auto &ce_w = embedding_r(w);
            ogdf::edge e_sk_l = skel_edge_at(*spqr, left_into_w, w);
            ogdf::edge e_sk_r = skel_edge_at(*spqr, right_into_w, w);
            return memo_return(share_faces(faces_of(ce_w, e_sk_l), faces_of(ce_w, e_sk_r)));
        }

        if (left_into_w || right_into_w)
        {
            ogdf::edge in_e = left_into_w ? left_into_w : right_into_w;
            ogdf::node ep = left_into_w ? bB : aB;
            return memo_return(endpoint_mask_at(w, in_e, ep) != 0);
        }

        return memo_return(cofacial_at_node(w, aB, bB));
    }

    void PlanarSubgraph::Impl::reset_blocks(const ogdf::Graph &bc_tree)
    {
        owned_blocks.clear();
        owned_blocks.reserve(bc_tree.numberOfNodes());
        blocks = std::make_unique<ogdf::NodeArray<BlockInfo *>>(bc_tree, nullptr);
    }

    BlockInfo &PlanarSubgraph::Impl::get_or_build_block(const BCTreeX &bc, ogdf::node vB)
    {
        BlockInfo *p = (*blocks)[vB];
        if (p)
        {
            return *p;
        }
        owned_blocks.emplace_back(std::make_unique<BlockInfo>(bc, vB));
        p = owned_blocks.back().get();
        (*blocks)[vB] = p;
        return *p;
    }

    bool PlanarSubgraph::Impl::can_add_along_bc(const BCTreeX &bc, ogdf::node uG, ogdf::node vG)
    {
        ogdf::node uB = bc.bcproper(uG);
        ogdf::node vB = bc.bcproper(vG);
        if (!uB || !vB)
        {
            return true;
        }

        ogdf::node w = bc.findNCA(uB, vB);
        if (!w)
        {
            return true;
        }

        auto process_branch = [&](ogdf::node start, ogdf::node lca, ogdf::node end_g,
                                  bool is_left) -> std::pair<bool, ogdf::node>
        {
            ogdf::node last_c = nullptr;
            ogdf::node cur = start;

            if (bc.typeOfBNode(cur) != ogdf::BCTree::BNodeType::BComp)
            {
                last_c = cur;
                cur = bc.parent(cur);
            }

            while (cur && cur != lca)
            {
                ogdf::node pC = (last_c == lca) ? last_c : bc.parent(cur);

                ogdf::node attach_left = bc.cutVertex(pC, cur);
                ogdf::node attach_right =
                    (!last_c || last_c == lca) ? bc.repVertex(end_g, cur) : bc.cutVertex(last_c, cur);
                if (is_left)
                {
                    std::swap(attach_left, attach_right);
                }

                if (!get_or_build_block(bc, cur).block_linkable(attach_left, attach_right))
                {
                    return {false, last_c ? last_c : pC};
                }

                if (pC == lca)
                {
                    return {true, pC};
                }
                last_c = pC;
                cur = bc.parent(pC);
            }
            return {true, last_c};
        };

        auto [ok_l, left_c] = process_branch(uB, w, uG, true);
        if (!ok_l)
        {
            return false;
        }
        auto [ok_r, right_c] = process_branch(vB, w, vG, false);
        if (!ok_r)
        {
            return false;
        }

        if (bc.typeOfBNode(w) == ogdf::BCTree::BNodeType::BComp)
        {
            ogdf::node lattach = (uB == w) ? bc.repVertex(uG, w) : (left_c ? bc.cutVertex(left_c, w) : nullptr);
            ogdf::node rattach = (vB == w) ? bc.repVertex(vG, w) : (right_c ? bc.cutVertex(right_c, w) : nullptr);
            if (!get_or_build_block(bc, w).block_linkable(lattach, rattach))
            {
                return false;
            }
        }
        return true;
    }

    PlanarSubgraph::Impl::Impl(const std::size_t num_verts,
                               const std::vector<Edge> &edges_all_in,
                               const std::vector<std::uint8_t> &edges_added_init)
        : num_verts(num_verts),
          edges_all(edges_all_in),
          edges_added(edges_all_in.size(), 0u)
    {

        if (edges_added_init.size() != edges_all_in.size())
        {
            throw std::invalid_argument("edges_added must match edges_all length");
        }

        for (const auto &[u, v] : edges_all)
        {
            if (u >= num_verts || v >= num_verts)
            {
                throw std::out_of_range("edge endpoint index out of range");
            }
            if (u == v)
            {
                throw std::invalid_argument("self edge not allowed");
            }
        }

        for (size_t i = 0; i < edges_added_init.size(); ++i)
        {
            edges_added[i] = edges_added_init[i] ? 1u : 0u;
        }

        v_g.resize(static_cast<size_t>(num_verts), nullptr);
        for (size_t i = 0; i < num_verts; ++i)
        {
            v_g[static_cast<size_t>(i)] = G.newNode();
        }
        cur_edges.resize(edges_all.size(), nullptr);

        std::lock_guard<std::mutex> lock(mu);
        recalculate_addable(edges_added);
    }

    PlanarSubgraph::Impl::~Impl() = default;

    void PlanarSubgraph::Impl::set(const std::size_t edge_id, const bool present)
    {
        std::lock_guard<std::mutex> lock(mu);
        size_t i = static_cast<size_t>(edge_id);
        if (i >= edges_added.size())
        {
            throw std::out_of_range("edge_id out of range");
        }

        if (present == !!edges_added[i])
        {
            return;
        }

        edges_added[i] = present ? 1u : 0u;
        recalculate_addable(edges_added);
    }

    std::vector<std::uint8_t> PlanarSubgraph::Impl::query() const
    {
        std::shared_ptr<const std::vector<std::uint8_t>> snap;
        {
            std::lock_guard<std::mutex> lock(mu);
            snap = addable_snap;
        }
        std::vector<std::uint8_t> out;
        if (!snap)
        {
            return out;
        }
        out.insert(out.end(), snap->begin(), snap->end());
        return out;
    }

    void PlanarSubgraph::Impl::recalculate_addable(const std::vector<std::uint8_t> &added_snapshot)
    {
        for (size_t i = 0; i < edges_all.size(); ++i)
        {
            const bool want = added_snapshot[i] != 0;
            auto [u, v] = edges_all[i];
            const bool is_loop = (u == v);
            ogdf::edge &eh = cur_edges[i];
            if (eh && (!want || is_loop))
            {
                G.delEdge(eh);
                eh = nullptr;
            }
            else if (!eh && want && !is_loop)
            {
                eh = G.newEdge(v_g[static_cast<size_t>(u)], v_g[static_cast<size_t>(v)]);
            }
        }

        if (!G.numberOfNodes())
        {
            auto empty = std::make_shared<std::vector<std::uint8_t>>(edges_all.size(), 0u);
            std::shared_ptr<const std::vector<std::uint8_t>> empty_const = empty;
            addable_snap = empty_const;
            return;
        }

        ogdf::NodeArray<int> comp(G, -1);
        ogdf::connectedComponents(G, comp);

        BCTreeX bc(G, true);
        const ogdf::Graph &bc_tree = bc.bcTree();

        reset_blocks(bc_tree);

        auto next_addable = std::make_shared<std::vector<std::uint8_t>>(edges_all.size(), 0u);
        for (size_t i = 0; i < edges_all.size(); ++i)
        {
            auto [ui, vi] = edges_all[i];
            ogdf::node u = v_g[static_cast<size_t>(ui)], v = v_g[static_cast<size_t>(vi)];

            if (added_snapshot[i])
            {
                (*next_addable)[i] = 0;
                continue;
            }
            if (ui == vi || comp[u] != comp[v])
            {
                (*next_addable)[i] = 1;
                continue;
            }

            (*next_addable)[i] = can_add_along_bc(bc, u, v);
        }

        std::shared_ptr<const std::vector<std::uint8_t>> pub = next_addable;
        addable_snap = pub;
    }

    PlanarSubgraph::PlanarSubgraph(const std::size_t num_verts,
                                   const std::vector<Edge> &edges_all_in,
                                   const std::vector<std::uint8_t> &edges_added_init)
        : impl(std::make_unique<Impl>(num_verts, edges_all_in, edges_added_init)) {}

    PlanarSubgraph::PlanarSubgraph(PlanarSubgraph &&) noexcept = default;
    PlanarSubgraph &PlanarSubgraph::operator=(PlanarSubgraph &&) noexcept = default;
    PlanarSubgraph::~PlanarSubgraph() = default;

    void PlanarSubgraph::set(const std::size_t edge_id, const bool present)
    {
        impl->set(edge_id, present);
    }

    std::vector<std::uint8_t> PlanarSubgraph::query() const
    {
        return impl->query();
    }

} // namespace graph
