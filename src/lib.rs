//! Directed-graph structural metrics — value-exact port of three
//! `networkx` 3.6.1 functions:
//!
//! - [`overall_reciprocity`] mirrors `nx.overall_reciprocity`
//! - [`node_reciprocity`] mirrors `nx.reciprocity(G, nodes=all)`
//! - [`flow_hierarchy`] mirrors `nx.flow_hierarchy`
//!
//! Input is a directed edge list (`u v` = u→v, whitespace separated, string
//! labels). Text from the first `#` onward is a comment, blank lines are
//! skipped, and parallel edges are deduplicated — the simple digraph over the
//! *edge-list node set*. Self-loops
//! are kept: a node with a self-loop is its own predecessor and successor, so
//! `nx.reciprocity` counts the loop as fully reciprocated. Isolated nodes are
//! unrepresentable from an edge list, so the per-node `None` branch of
//! `nx.reciprocity` (which fires only for a degree-0 node) is unreachable
//! through the CLI; it is still honoured by [`node_reciprocity`].
//!
//! References: NetworkX (BSD-3-Clause); Luo, J. & Magee, C.L. (2011),
//! "Detecting evolving patterns of self-organizing networks by flow hierarchy
//! measurement", Complexity 16(6):53-61, DOI 10.1002/cplx.20368.

use std::collections::{HashMap, HashSet};

use serde::Serialize;

/// Simple directed graph over interned integer node ids `0..n`, holding both
/// successor and predecessor adjacency so per-node reciprocity avoids any
/// `HashMap` lookup in the hot loop.
pub struct DiGraph {
    idx_to_node: Vec<String>,
    succ: Vec<Vec<usize>>,
    pred: Vec<Vec<usize>>,
    /// Distinct directed edges `(u, v)`, u→v, deduped; self-loops kept.
    edges: HashSet<(usize, usize)>,
}

impl DiGraph {
    fn intern(&mut self, name: &str, table: &mut HashMap<String, usize>) -> usize {
        if let Some(&idx) = table.get(name) {
            return idx;
        }
        let idx = self.idx_to_node.len();
        table.insert(name.to_owned(), idx);
        self.idx_to_node.push(name.to_owned());
        self.succ.push(Vec::new());
        self.pred.push(Vec::new());
        idx
    }

    #[must_use]
    pub fn node_count(&self) -> usize {
        self.idx_to_node.len()
    }

    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    #[must_use]
    pub fn label(&self, idx: usize) -> &str {
        &self.idx_to_node[idx]
    }
}

/// Parse a whitespace-delimited directed edge list (`u v` = u→v). Text from the
/// first `#` onward is a comment and dropped before tokenising; blank lines are
/// skipped; parallel edges deduped; self-loops kept — the simple digraph
/// `nx.parse_edgelist(create_using=nx.DiGraph)` yields.
#[must_use]
pub fn parse_edge_list(input: &str) -> DiGraph {
    let mut g = DiGraph {
        idx_to_node: Vec::new(),
        succ: Vec::new(),
        pred: Vec::new(),
        edges: HashSet::new(),
    };
    let mut table = HashMap::new();

    for line in input.lines() {
        // nx.parse_edgelist strips a '#' comment anywhere in the line before tokenising.
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let (Some(u), Some(v)) = (parts.next(), parts.next()) else {
            continue;
        };
        let ui = g.intern(u, &mut table);
        let vi = g.intern(v, &mut table);
        if g.edges.insert((ui, vi)) {
            g.succ[ui].push(vi);
            g.pred[vi].push(ui);
        }
    }
    g
}

/// `nx.overall_reciprocity`: `2 * (m_dir - m_undir) / m_dir`, where `m_undir`
/// is the number of edges after collapsing direction. The number of reciprocal
/// directed edges equals `2 * (m_dir - m_undir)`; NetworkX evaluates the
/// division in exactly this order.
#[must_use]
pub fn overall_reciprocity(g: &DiGraph) -> f64 {
    let n_all = g.edges.len();
    assert!(
        n_all != 0,
        "overall_reciprocity not defined for empty graphs"
    );

    let mut undirected: HashSet<(usize, usize)> = HashSet::with_capacity(n_all);
    for &(u, v) in &g.edges {
        undirected.insert(if u <= v { (u, v) } else { (v, u) });
    }
    let n_undir = undirected.len();
    let n_overlap = (n_all - n_undir) * 2;
    n_overlap as f64 / n_all as f64
}

/// One node's reciprocity: `Some(label)` with its value, `None` for a degree-0
/// (isolated) node, matching `nx._reciprocity_single_node`.
#[derive(Debug, Clone, Serialize)]
pub struct NodeReciprocity {
    pub node: String,
    /// `None` for an isolated node (unreachable from edge-list input).
    pub reciprocity: Option<f64>,
}

/// `nx.reciprocity(G, nodes=all)`: per node `2 * |pred ∩ succ| / (|pred| + |succ|)`,
/// with predecessors and successors taken as *sets* (distinct neighbours).
/// A degree-0 node yields `None`. Output is sorted by node label to give a
/// deterministic ordering (NetworkX iterates in insertion order).
#[must_use]
pub fn node_reciprocity(g: &DiGraph) -> Vec<NodeReciprocity> {
    let n = g.node_count();
    let mut out = Vec::with_capacity(n);
    let mut pred_set: HashSet<usize> = HashSet::new();

    for v in 0..n {
        pred_set.clear();
        pred_set.extend(g.pred[v].iter().copied());
        let n_pred = pred_set.len();
        let succ = &g.succ[v];
        let n_succ = succ.len();
        let n_total = n_pred + n_succ;

        let reciprocity = if n_total == 0 {
            None
        } else {
            let overlap = succ.iter().filter(|s| pred_set.contains(s)).count();
            Some(2.0 * overlap as f64 / n_total as f64)
        };
        out.push(NodeReciprocity {
            node: g.idx_to_node[v].clone(),
            reciprocity,
        });
    }

    out.sort_by(|a, b| a.node.cmp(&b.node));
    out
}

/// `nx.flow_hierarchy`: `1 - (edges inside strongly connected components) / m`.
/// An edge lies on a cycle iff both endpoints share an SCC, so the numerator is
/// the count of directed edges whose head and tail belong to the same SCC.
/// SCCs are found with an iterative (stack-based) Tarjan pass — `O(n + m)`.
#[must_use]
pub fn flow_hierarchy(g: &DiGraph) -> f64 {
    let m = g.edges.len();
    assert!(m != 0, "flow_hierarchy not applicable to empty graphs");

    let comp = tarjan_scc(g);
    let mut in_cycle = 0usize;
    for &(u, v) in &g.edges {
        if comp[u] == comp[v] {
            in_cycle += 1;
        }
    }
    1.0 - in_cycle as f64 / m as f64
}

/// Iterative Tarjan strongly-connected-components. Returns a component id per
/// node; nodes sharing an id are strongly connected. Non-recursive to stay safe
/// on deep graphs.
fn tarjan_scc(g: &DiGraph) -> Vec<usize> {
    let n = g.node_count();
    let mut index = vec![usize::MAX; n];
    let mut low = vec![0usize; n];
    let mut on_stack = vec![false; n];
    let mut comp = vec![usize::MAX; n];
    let mut stack: Vec<usize> = Vec::new();
    let mut next_index = 0usize;
    let mut next_comp = 0usize;

    // Explicit DFS stack: (node, position in its successor list).
    let mut work: Vec<(usize, usize)> = Vec::new();

    for start in 0..n {
        if index[start] != usize::MAX {
            continue;
        }
        work.push((start, 0));

        while let Some(&(v, i)) = work.last() {
            if i == 0 {
                index[v] = next_index;
                low[v] = next_index;
                next_index += 1;
                stack.push(v);
                on_stack[v] = true;
            }

            if i < g.succ[v].len() {
                let w = g.succ[v][i];
                work.last_mut().unwrap().1 += 1;
                if index[w] == usize::MAX {
                    work.push((w, 0));
                } else if on_stack[w] {
                    low[v] = low[v].min(index[w]);
                }
            } else {
                if low[v] == index[v] {
                    loop {
                        let w = stack.pop().unwrap();
                        on_stack[w] = false;
                        comp[w] = next_comp;
                        if w == v {
                            break;
                        }
                    }
                    next_comp += 1;
                }
                work.pop();
                if let Some(&(parent, _)) = work.last() {
                    low[parent] = low[parent].min(low[v]);
                }
            }
        }
    }

    comp
}

/// Which metric to compute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metric {
    OverallReciprocity,
    NodeReciprocity,
    FlowHierarchy,
}

/// Serialisable result payload for the `--json` envelope.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum MetricResult {
    Scalar {
        metric: &'static str,
        value: f64,
    },
    PerNode {
        metric: &'static str,
        nodes: Vec<NodeReciprocity>,
    },
}

/// Compute the requested metric on a parsed graph (the compute-only entry
/// point benches and tests call).
#[must_use]
pub fn compute(g: &DiGraph, metric: Metric) -> MetricResult {
    match metric {
        Metric::OverallReciprocity => MetricResult::Scalar {
            metric: "overall-reciprocity",
            value: overall_reciprocity(g),
        },
        Metric::FlowHierarchy => MetricResult::Scalar {
            metric: "flow-hierarchy",
            value: flow_hierarchy(g),
        },
        Metric::NodeReciprocity => MetricResult::PerNode {
            metric: "node-reciprocity",
            nodes: node_reciprocity(g),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn edge_label_set(g: &DiGraph) -> BTreeSet<(String, String)> {
        g.edges
            .iter()
            .map(|&(u, v)| (g.label(u).to_owned(), g.label(v).to_owned()))
            .collect()
    }

    #[test]
    fn inline_hash_comment_matches_comment_free_graph() {
        // A '#' anywhere in a line begins a comment: "1 2#c" is the edge 1→2,
        // and "0 #x" collapses to the lone token "0" (no target) so it is
        // skipped — exactly as nx.parse_edgelist treats these lines.
        let with_comments = parse_edge_list("0 1\n1 2#c\n2 3\n0 #x\n# whole line\n");
        let clean = parse_edge_list("0 1\n1 2\n2 3\n");

        assert_eq!(edge_label_set(&with_comments), edge_label_set(&clean));
        assert_eq!(with_comments.node_count(), clean.node_count());
        assert_eq!(with_comments.edge_count(), clean.edge_count());
    }
}
