use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

use rsomics_digraph_metrics::{
    flow_hierarchy, node_reciprocity, overall_reciprocity, parse_edge_list, DiGraph,
};

/// Deterministic directed G(n, p) edge list via a splitmix64 stream, so the
/// bench fixture is fixed without pulling in an RNG crate.
fn gnp_edge_list(n: usize, p: f64, seed: u64) -> String {
    let mut state = seed;
    let mut next = || {
        state = state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    };
    let mut out = String::new();
    let thresh = (p * (u64::MAX as f64)) as u64;
    for u in 0..n {
        for v in 0..n {
            if u != v && next() < thresh {
                out.push_str(&u.to_string());
                out.push(' ');
                out.push_str(&v.to_string());
                out.push('\n');
            }
        }
    }
    out
}

fn bench(c: &mut Criterion) {
    let text = gnp_edge_list(2000, 0.004, 20260701);
    let g: DiGraph = parse_edge_list(&text);
    eprintln!("bench graph: n={} m={}", g.node_count(), g.edge_count());

    c.bench_function("overall_reciprocity", |b| {
        b.iter(|| black_box(overall_reciprocity(black_box(&g))))
    });
    c.bench_function("node_reciprocity", |b| {
        b.iter(|| black_box(node_reciprocity(black_box(&g))))
    });
    c.bench_function("flow_hierarchy", |b| {
        b.iter(|| black_box(flow_hierarchy(black_box(&g))))
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
