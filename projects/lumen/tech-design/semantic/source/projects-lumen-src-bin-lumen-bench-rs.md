---
id: projects-lumen-src-bin-lumen-bench-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: competitor-performance
    role: primary
    claim: "depth-invariant-filter-sort-pagination"
    coverage: partial
    rationale: "The lumen-bench sorted_page_deep cell is the local benchmark executable for the depth-invariant filter/sort pagination claim."
---

# Standardized projects/lumen/src/bin/lumen-bench.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/src/bin/lumen-bench.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// @spec projects/lumen/tech-design/logic/gate-the-filter-sort-deep-page-chain-bench-cell-pg-competitive-p.md#logic
use std::collections::BTreeMap;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use lumen::storage::{Engine, MAX_INDEX_ITEMS};
use lumen::types::{
    CreateCollectionRequest, FieldSpec, FieldType, FieldValue, IndexItem, IndexRequest, QueryNode,
    RangeQuery, SearchRequest, SortMissing, SortOrder, SortSpec, TermQuery,
};

const DEFAULT_DOCUMENTS: usize = 20_000;
const DEFAULT_PAGE_SIZE: u32 = 100;
const DEFAULT_QUERIES: usize = 200;
const SORTED_PAGE_BUDGET_US: u128 = 5_000;

#[derive(Parser)]
#[command(name = "lumen-bench", version, about = "Lumen local benchmark runner")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run one or more benchmark cells.
    Run(RunArgs),
}

#[derive(Parser)]
struct RunArgs {
    /// Comma-separated cell list. Supported: sorted_page_deep, bool_filter.
    #[arg(long, default_value = "sorted_page_deep")]
    types: String,
    /// Compatibility knob used by vat runner specs; accepted but not interpreted yet.
    #[arg(long, default_value = "s")]
    tiers: String,
    /// Query/page sample cap. For sorted_page_deep this caps measured pages near depth.
    #[arg(long, default_value_t = DEFAULT_QUERIES)]
    queries: usize,
    /// Number of documents in the synthetic corpus.
    #[arg(long, default_value_t = DEFAULT_DOCUMENTS)]
    documents: usize,
    /// Page size for sorted cursor walks.
    #[arg(long, default_value_t = DEFAULT_PAGE_SIZE)]
    page_size: u32,
}

#[derive(Debug)]
struct BenchReport {
    cell: &'static str,
    documents: usize,
    pages_walked: usize,
    measured_pages: usize,
    p50_us: u128,
    p99_us: u128,
    min_us: u128,
    max_us: u128,
    budget_us: u128,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Run(args) => run(args),
    }
}

fn run(args: RunArgs) -> Result<()> {
    if args.documents == 0 {
        bail!("--documents must be > 0");
    }
    if args.page_size == 0 {
        bail!("--page-size must be > 0");
    }
    let _tiers = &args.tiers;
    for cell in parse_types(&args.types)? {
        match cell {
            "sorted_page_deep" => print_report(run_sorted_page_deep(&args)?),
            "bool_filter" => print_report(run_bool_filter(&args)?),
            _ => unreachable!("parse_types only returns supported cells"),
        }
    }
    Ok(())
}

fn parse_types(raw: &str) -> Result<Vec<&'static str>> {
    let mut cells = Vec::new();
    for token in raw.split(',') {
        let cell = token.trim();
        if cell.is_empty() {
            continue;
        }
        let known = match cell {
            "sorted_page_deep" => "sorted_page_deep",
            "bool_filter" => "bool_filter",
            other => bail!("unknown bench cell `{other}`; supported: sorted_page_deep,bool_filter"),
        };
        cells.push(known);
    }
    if cells.is_empty() {
        bail!("--types did not name any bench cells");
    }
    Ok(cells)
}

fn print_report(report: BenchReport) {
    println!(
        "cell={} documents={} pages_walked={} measured_pages={} min_us={} p50_us={} p99_us={} max_us={} budget_us={} status=pass",
        report.cell,
        report.documents,
        report.pages_walked,
        report.measured_pages,
        report.min_us,
        report.p50_us,
        report.p99_us,
        report.max_us,
        report.budget_us
    );
}

fn run_sorted_page_deep(args: &RunArgs) -> Result<BenchReport> {
    let engine = build_corpus(args.documents)?;
    let depth = args.documents / 2;
    let page_size = args.page_size as usize;
    let target_page = (depth / page_size).max(1);
    let measure_window = args.queries.max(1).min(target_page + 1);
    let measure_from = target_page + 1 - measure_window;
    let mut cursor = None;
    let mut samples = Vec::with_capacity(measure_window);

    for page_idx in 0..=target_page {
        let mut req = sorted_page_request(args.page_size);
        req.cursor = cursor.take();
        let started = Instant::now();
        let resp = engine
            .search("docs", req)
            .with_context(|| format!("sorted_page_deep page {page_idx}"))?;
        let elapsed = started.elapsed().as_micros();
        if page_idx >= measure_from {
            samples.push(elapsed);
        }
        if resp.hits.is_empty() {
            bail!("sorted_page_deep exhausted before target depth {depth}");
        }
        cursor = resp.cursor;
    }

    let report = summarize("sorted_page_deep", args.documents, target_page + 1, samples);
    if report.p99_us > SORTED_PAGE_BUDGET_US {
        bail!(
            "sorted_page_deep p99 {}us exceeds budget {}us",
            report.p99_us,
            SORTED_PAGE_BUDGET_US
        );
    }
    Ok(report)
}

fn run_bool_filter(args: &RunArgs) -> Result<BenchReport> {
    let engine = build_corpus(args.documents)?;
    let reps = args.queries.max(1);
    let mut samples = Vec::with_capacity(reps);
    for _ in 0..reps {
        let started = Instant::now();
        let resp = engine
            .search(
                "docs",
                SearchRequest {
                    query: QueryNode::And(vec![
                        QueryNode::Term(TermQuery {
                            field: "city".into(),
                            value: FieldValue::String("taipei".into()),
                        }),
                        QueryNode::Range(RangeQuery {
                            field: "age".into(),
                            gt: None,
                            gte: Some(30.0),
                            lt: Some(40.0),
                            lte: None,
                        }),
                    ]),
                    limit: 20,
                    cursor: None,
                    sort: None,
                    track_total: false,
                    collapse: None,
                },
            )
            .context("bool_filter search")?;
        if resp.hits.is_empty() {
            bail!("bool_filter returned no hits");
        }
        samples.push(started.elapsed().as_micros());
    }
    Ok(summarize("bool_filter", args.documents, reps, samples))
}

fn summarize(
    cell: &'static str,
    documents: usize,
    pages_walked: usize,
    mut samples: Vec<u128>,
) -> BenchReport {
    samples.sort_unstable();
    let percentile = |q: f64| -> u128 {
        let idx = (((samples.len() - 1) as f64) * q).round() as usize;
        samples[idx]
    };
    BenchReport {
        cell,
        documents,
        pages_walked,
        measured_pages: samples.len(),
        min_us: samples[0],
        p50_us: percentile(0.50),
        p99_us: percentile(0.99),
        max_us: *samples.last().expect("non-empty samples"),
        budget_us: SORTED_PAGE_BUDGET_US,
    }
}

fn sorted_page_request(limit: u32) -> SearchRequest {
    SearchRequest {
        query: QueryNode::Range(RangeQuery {
            field: "age".into(),
            gt: None,
            gte: Some(0.0),
            lt: None,
            lte: None,
        }),
        limit,
        cursor: None,
        sort: Some(vec![SortSpec {
            field: "age".into(),
            order: SortOrder::Asc,
            missing: SortMissing::Exclude,
        }]),
        track_total: false,
        collapse: None,
    }
}

fn build_corpus(n: usize) -> Result<Engine> {
    let engine = Engine::new();
    let mut fields = BTreeMap::new();
    fields.insert("city".into(), spec(FieldType::Keyword));
    fields.insert("age".into(), spec(FieldType::Number));
    engine.create_collection("docs", CreateCollectionRequest { fields })?;

    let max_docs_per_batch = (MAX_INDEX_ITEMS / 2).max(1);
    let mut start = 0usize;
    while start < n {
        let end = (start + max_docs_per_batch).min(n);
        let mut items = Vec::with_capacity((end - start) * 2);
        for i in start..end {
            let city = if i % 3 == 0 { "taipei" } else { "tokyo" };
            items.push(IndexItem {
                external_id: format!("doc-{i:06}"),
                field: "city".into(),
                value: FieldValue::String(city.into()),
                version: None,
            });
            items.push(IndexItem {
                external_id: format!("doc-{i:06}"),
                field: "age".into(),
                value: FieldValue::Number((i % 1_000_000) as f64),
                version: None,
            });
        }
        engine.index(
            "docs",
            IndexRequest {
                items,
                request_id: None,
            },
        )?;
        start = end;
    }
    Ok(engine)
}

fn spec(field_type: FieldType) -> FieldSpec {
    FieldSpec {
        field_type,
        analyzer: None,
        multi: None,
        dim: None,
        metric: None,
        backend: None,
        quantize: None,
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/src/bin/lumen-bench.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/src/bin/lumen-bench.rs` captured for the sorted_page_deep benchmark cell.
```
