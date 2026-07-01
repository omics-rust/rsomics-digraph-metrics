# rsomics-digraph-metrics

Directed-graph structural metrics — a value-exact Rust port of three NetworkX
functions, in one cohesive CLI selected with `--metric`:

| `--metric`             | NetworkX                        | Output              |
|------------------------|---------------------------------|---------------------|
| `overall-reciprocity`  | `nx.overall_reciprocity(G)`     | a single float      |
| `node-reciprocity`     | `nx.reciprocity(G)` (all nodes) | `node<TAB>value`/line |
| `flow-hierarchy`       | `nx.flow_hierarchy(G)`          | a single float      |

`overall-reciprocity` is the default.

## Install

```
cargo install rsomics-digraph-metrics
```

## Usage

Input is a **directed** edge list on stdin (or a file path): one `u v` per line
meaning `u → v`, whitespace separated, with string node labels.

```
printf '0 1\n1 0\n1 2\n2 3\n3 2\n' | rsomics-digraph-metrics
# 0.8

printf '0 1\n1 0\n1 2\n2 3\n3 2\n' | rsomics-digraph-metrics --metric flow-hierarchy
# 0.19999999999999996

printf '0 1\n1 0\n1 2\n2 3\n3 2\n' | rsomics-digraph-metrics --metric node-reciprocity
# 0	1
# 1	0.6666666666666666
# 2	0.6666666666666666
# 3	1
```

`--json` wraps the result in the standard rsomics envelope.

### Definitions (matching NetworkX exactly)

- **overall reciprocity** = `2·(m_dir − m_undir) / m_dir`, i.e. the fraction of
  directed edges that have a reciprocal partner. `m_undir` is the edge count
  after collapsing direction. NetworkX divides in exactly this order.
- **node reciprocity** for a node `v` = `2·|pred(v) ∩ succ(v)| / (|pred(v)| + |succ(v)|)`,
  where `pred`/`succ` are the *sets* of distinct in-/out-neighbours. A degree-0
  (isolated) node is `None` in NetworkX; this port emits `nan` for that case,
  but see the input contract below — an edge list cannot produce one.
- **flow hierarchy** = `1 − (edges inside non-trivial strongly connected
  components) / m`, the fraction of edges not on any cycle. Strongly connected
  components are found with an iterative Tarjan pass in `O(n + m)`.

### Input contract (edge-list node set)

The node set is exactly the labels that appear in the edge list. Consequently:

- **Isolated nodes are unrepresentable.** Every node has at least one incident
  edge, so `node-reciprocity` never emits the `None`/`nan` branch through this
  CLI. `flow-hierarchy` and `overall-reciprocity` require ≥ 1 edge, matching
  NetworkX (which raises on an empty graph).
- **Parallel edges are deduplicated** and **self-loops are dropped** at parse.
  The graph is therefore the *simple* digraph over the edge-list node set —
  what `nx.parse_edgelist(..., create_using=nx.DiGraph)` yields for simple
  input. (NetworkX itself would count a self-loop toward `reciprocity`/
  `flow_hierarchy`; here they are removed at the boundary by design.)
- `#` comments and blank lines are ignored.

## Origin

This crate is an independent Rust reimplementation of three
[NetworkX](https://networkx.org) directed-graph metrics
(`overall_reciprocity`, `reciprocity`, `flow_hierarchy`), matching their exact
numerator/denominator construction and division order for value-exact
(≤ 1e-12) agreement. NetworkX is pure Python under the **BSD-3-Clause** license,
which permits reading and citing its source; the formulas above follow the
NetworkX 3.6.1 implementation.

Flow hierarchy follows:

- Luo, J.; Magee, C.L. (2011). *Detecting evolving patterns of self-organizing
  networks by flow hierarchy measurement.* Complexity 16(6):53–61.
  DOI: [10.1002/cplx.20368](https://doi.org/10.1002/cplx.20368)

Goldens are generated once from NetworkX 3.6.1 and hardcoded in the test suite;
no Python runs at test time.

License: MIT OR Apache-2.0. Upstream credit: NetworkX (BSD-3-Clause).
