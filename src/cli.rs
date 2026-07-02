use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use rsomics_common::{run, CommonFlags, RsomicsError, ToolMeta};

use rsomics_digraph_metrics::{compute, parse_edge_list, Metric, MetricResult};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum MetricArg {
    /// Fraction of edges with a reciprocal partner (single float).
    OverallReciprocity,
    /// Per-node `2·|pred∩succ|/(|pred|+|succ|)` (`node value` per line).
    NodeReciprocity,
    /// Fraction of edges not on a cycle (single float).
    FlowHierarchy,
}

impl From<MetricArg> for Metric {
    fn from(a: MetricArg) -> Self {
        match a {
            MetricArg::OverallReciprocity => Metric::OverallReciprocity,
            MetricArg::NodeReciprocity => Metric::NodeReciprocity,
            MetricArg::FlowHierarchy => Metric::FlowHierarchy,
        }
    }
}

/// Directed-graph structural metrics (`networkx` reciprocity / flow hierarchy).
///
/// Reads a directed edge list (`u v` = u→v, one per line; `#` comments and
/// blank lines skipped; string node labels; parallel edges deduplicated,
/// self-loops kept, giving the simple digraph over the edge-list node set).
/// Isolated nodes are unrepresentable from an edge list.
#[derive(Parser, Debug)]
#[command(name = "rsomics-digraph-metrics", version, about, long_about = None)]
pub struct Cli {
    /// Directed edge list; `-` or omitted reads stdin.
    #[arg(value_name = "EDGES")]
    pub edges: Option<PathBuf>,

    /// Which structural metric to compute.
    #[arg(long, value_enum, default_value_t = MetricArg::OverallReciprocity)]
    pub metric: MetricArg,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn run(self) -> ExitCode {
        let common = self.common.clone();
        run(&common, META, || {
            let mut input = String::new();
            match &self.edges {
                Some(p) if p.as_os_str() != "-" => {
                    File::open(p)
                        .map_err(RsomicsError::Io)?
                        .read_to_string(&mut input)
                        .map_err(RsomicsError::Io)?;
                }
                _ => {
                    io::stdin()
                        .lock()
                        .read_to_string(&mut input)
                        .map_err(RsomicsError::Io)?;
                }
            }
            let g = parse_edge_list(&input);
            if g.edge_count() == 0 {
                return Err(RsomicsError::InvalidInput(
                    "edge list is empty; metrics are undefined for a graph with no edges".into(),
                ));
            }

            let result = compute(&g, self.metric.into());
            if !common.json {
                match &result {
                    MetricResult::Scalar { value, .. } => println!("{value}"),
                    MetricResult::PerNode { nodes, .. } => {
                        for nr in nodes {
                            match nr.reciprocity {
                                Some(v) => println!("{}\t{v}", nr.node),
                                None => println!("{}\tnan", nr.node),
                            }
                        }
                    }
                }
            }
            Ok(result)
        })
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn cli_debug_assert() {
        super::Cli::command().debug_assert();
    }
}
