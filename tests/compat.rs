//! Value-exact compatibility against networkx 3.6.1. All oracle values are
//! captured once at build time and hardcoded here; no subprocess runs at test
//! time. Edge lists are committed under `tests/golden/` and read via
//! `include_str!`, so the graph the Rust parser sees is byte-identical to the
//! one the oracle saw.

use rsomics_digraph_metrics::{
    flow_hierarchy, node_reciprocity, overall_reciprocity, parse_edge_list,
};

const TOL: f64 = 1e-12;

fn assert_close(got: f64, want: f64, ctx: &str) {
    assert!(
        (got - want).abs() <= TOL,
        "{ctx}: got {got:.17e}, want {want:.17e}, |Δ|={:.3e}",
        (got - want).abs()
    );
}

/// (label, expected reciprocity or NaN for nx `None`).
fn assert_nodes(input: &str, expected: &[(&str, f64)], ctx: &str) {
    let g = parse_edge_list(input);
    let got = node_reciprocity(&g);
    assert_eq!(got.len(), expected.len(), "{ctx}: node count");
    for (nr, (label, want)) in got.iter().zip(expected.iter()) {
        assert_eq!(&nr.node, label, "{ctx}: label order");
        match nr.reciprocity {
            Some(v) => {
                assert!(!want.is_nan(), "{ctx}: {label} nx=None but got {v}");
                assert_close(v, *want, &format!("{ctx}:{label}"));
            }
            None => assert!(want.is_nan(), "{ctx}: {label} got None but nx={want}"),
        }
    }
}

// ---- hand graph: 0->1,1->0,1->2,2->3,3->2 ----------------------------------
const HAND: &str = include_str!("golden/hand.txt");

#[test]
fn hand_overall() {
    assert_close(
        overall_reciprocity(&parse_edge_list(HAND)),
        0.8,
        "hand overall",
    );
}

#[test]
fn hand_flow() {
    assert_close(
        flow_hierarchy(&parse_edge_list(HAND)),
        0.19999999999999996,
        "hand flow",
    );
}

#[test]
fn hand_nodes() {
    assert_nodes(
        HAND,
        &[
            ("0", 1.0),
            ("1", 0.6666666666666666),
            ("2", 0.6666666666666666),
            ("3", 1.0),
        ],
        "hand",
    );
}

// ---- pure DAG: 0->1,1->2,2->3,0->2 -----------------------------------------
const DAG: &str = include_str!("golden/dag.txt");

#[test]
fn dag_overall() {
    assert_close(
        overall_reciprocity(&parse_edge_list(DAG)),
        0.0,
        "dag overall",
    );
}

#[test]
fn dag_flow() {
    assert_close(flow_hierarchy(&parse_edge_list(DAG)), 1.0, "dag flow");
}

#[test]
fn dag_nodes() {
    assert_nodes(
        DAG,
        &[("0", 0.0), ("1", 0.0), ("2", 0.0), ("3", 0.0)],
        "dag",
    );
}

// ---- fully reciprocated: 0<->1, 1<->2 --------------------------------------
const FULL: &str = include_str!("golden/full.txt");

#[test]
fn full_overall() {
    assert_close(
        overall_reciprocity(&parse_edge_list(FULL)),
        1.0,
        "full overall",
    );
}

#[test]
fn full_flow() {
    assert_close(flow_hierarchy(&parse_edge_list(FULL)), 0.0, "full flow");
}

#[test]
fn full_nodes() {
    assert_nodes(FULL, &[("0", 1.0), ("1", 1.0), ("2", 1.0)], "full");
}

// ---- karate club, directed once by node order (a big DAG) ------------------
const KARATE: &str = include_str!("golden/karate_dir.txt");

#[test]
fn karate_scalars() {
    assert_close(
        overall_reciprocity(&parse_edge_list(KARATE)),
        0.0,
        "karate overall",
    );
    assert_close(flow_hierarchy(&parse_edge_list(KARATE)), 1.0, "karate flow");
}

#[test]
fn karate_all_nodes_zero() {
    let g = parse_edge_list(KARATE);
    for nr in node_reciprocity(&g) {
        assert_eq!(nr.reciprocity, Some(0.0), "karate {} reciprocity", nr.node);
    }
}

// ---- gnp(40, 0.15, seed=7, directed) ---------------------------------------
const GNP1: &str = include_str!("golden/gnp1.txt");

#[test]
fn gnp1_scalars() {
    assert_close(
        overall_reciprocity(&parse_edge_list(GNP1)),
        0.14457831325301204,
        "gnp1 overall",
    );
    assert_close(flow_hierarchy(&parse_edge_list(GNP1)), 0.0, "gnp1 flow");
}

#[test]
fn gnp1_nodes() {
    assert_nodes(
        GNP1,
        &[
            ("0", 0.35294117647058826),
            ("1", 0.0),
            ("10", 0.0),
            ("11", 0.10526315789473684),
            ("12", 0.3076923076923077),
            ("13", 0.16666666666666666),
            ("14", 0.0),
            ("15", 0.0),
            ("16", 0.42857142857142855),
            ("17", 0.2),
            ("18", 0.2),
            ("19", 0.3076923076923077),
            ("2", 0.0),
            ("20", 0.4),
            ("21", 0.0),
            ("22", 0.2222222222222222),
            ("23", 0.0),
            ("24", 0.0),
            ("25", 0.3),
            ("26", 0.18181818181818182),
            ("27", 0.2222222222222222),
            ("28", 0.15384615384615385),
            ("29", 0.0),
            ("3", 0.3),
            ("30", 0.23529411764705882),
            ("31", 0.14285714285714285),
            ("32", 0.0),
            ("33", 0.0),
            ("34", 0.0),
            ("35", 0.0),
            ("36", 0.2),
            ("37", 0.16666666666666666),
            ("38", 0.0),
            ("39", 0.0),
            ("4", 0.0),
            ("5", 0.0),
            ("6", 0.0),
            ("7", 0.15384615384615385),
            ("8", 0.14285714285714285),
            ("9", 0.14285714285714285),
        ],
        "gnp1",
    );
}

// ---- gnp(60, 0.08, seed=42, directed) --------------------------------------
const GNP2: &str = include_str!("golden/gnp2.txt");

#[test]
fn gnp2_scalars() {
    assert_close(
        overall_reciprocity(&parse_edge_list(GNP2)),
        0.06060606060606061,
        "gnp2 overall",
    );
    assert_close(flow_hierarchy(&parse_edge_list(GNP2)), 0.0, "gnp2 flow");
}

#[test]
fn gnp2_nodes() {
    assert_nodes(
        GNP2,
        &[
            ("0", 0.2222222222222222),
            ("1", 0.0),
            ("10", 0.16666666666666666),
            ("11", 0.2222222222222222),
            ("12", 0.0),
            ("13", 0.0),
            ("14", 0.0),
            ("15", 0.0),
            ("16", 0.25),
            ("17", 0.0),
            ("18", 0.0),
            ("19", 0.0),
            ("2", 0.125),
            ("20", 0.0),
            ("21", 0.0),
            ("22", 0.0),
            ("23", 0.2),
            ("24", 0.0),
            ("25", 0.0),
            ("26", 0.0),
            ("27", 0.0),
            ("28", 0.0),
            ("29", 0.0),
            ("3", 0.0),
            ("30", 0.0),
            ("31", 0.0),
            ("32", 0.18181818181818182),
            ("33", 0.0),
            ("34", 0.0),
            ("35", 0.0),
            ("36", 0.0),
            ("37", 0.0),
            ("38", 0.0),
            ("39", 0.0),
            ("4", 0.0),
            ("40", 0.0),
            ("41", 0.0),
            ("42", 0.2222222222222222),
            ("43", 0.0),
            ("44", 0.0),
            ("45", 0.15384615384615385),
            ("46", 0.0),
            ("47", 0.0),
            ("48", 0.23529411764705882),
            ("49", 0.0),
            ("5", 0.2),
            ("50", 0.0),
            ("51", 0.0),
            ("52", 0.0),
            ("53", 0.0),
            ("54", 0.2857142857142857),
            ("55", 0.3333333333333333),
            ("56", 0.0),
            ("57", 0.0),
            ("58", 0.0),
            ("59", 0.0),
            ("6", 0.0),
            ("7", 0.15384615384615385),
            ("8", 0.2),
            ("9", 0.0),
        ],
        "gnp2",
    );
}

// ---- structural edge cases -------------------------------------------------

/// Comments, blanks, a duplicate parallel edge, and a self-loop are all handled
/// the way the parser documents (dedup + self-loop drop); the resulting simple
/// digraph is the `hand` graph, so metrics are unchanged.
#[test]
fn parse_dedup_selfloop_comments() {
    let noisy = "# header\n0 1\n\n1 0\n1 0\n1 2\n2 2\n2 3\n3 2\n";
    let g = parse_edge_list(noisy);
    assert_eq!(g.edge_count(), 5, "self-loop + parallel dropped");
    assert_close(overall_reciprocity(&g), 0.8, "noisy overall");
    assert_close(flow_hierarchy(&g), 0.19999999999999996, "noisy flow");
}

/// A node with edges but no reciprocal overlap gets `2*0/n_total = 0.0`
/// (not `None`); `None` is reserved for a degree-0 node, which an edge list
/// cannot express. `x->y` gives x: succ={y},pred={} → 0/1=0; y: pred={x} → 0.
#[test]
fn node_no_overlap_is_zero_not_none() {
    let g = parse_edge_list("x y\n");
    let nodes = node_reciprocity(&g);
    for nr in &nodes {
        assert_eq!(
            nr.reciprocity,
            Some(0.0),
            "{} should be 0.0 not None",
            nr.node
        );
    }
}
