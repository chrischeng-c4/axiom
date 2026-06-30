---
id: projects-lumen-tests-lumen_bench_cli-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: competitor-performance
    role: primary
    claim: "depth-invariant-filter-sort-pagination"
    coverage: partial
    rationale: "The lumen-bench CLI smoke test proves the sorted_page_deep benchmark emits the latency fields required by the depth-invariant pagination gate."
---

# Standardized projects/lumen/tests/lumen_bench_cli.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/lumen/tests/lumen_bench_cli.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// @spec projects/lumen/tech-design/logic/gate-the-filter-sort-deep-page-chain-bench-cell-pg-competitive-p.md#unit-test
use std::process::Command;

#[test]
fn sorted_page_deep_bench_cli_reports_latency_fields() {
    let bin = env!("CARGO_BIN_EXE_lumen-bench");
    let output = Command::new(bin)
        .args([
            "run",
            "--types",
            "sorted_page_deep",
            "--documents",
            "1000",
            "--page-size",
            "50",
            "--queries",
            "10",
        ])
        .output()
        .expect("run lumen-bench");

    assert!(
        output.status.success(),
        "lumen-bench failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    assert!(stdout.contains("cell=sorted_page_deep"), "{stdout}");
    assert!(stdout.contains("p50_us="), "{stdout}");
    assert!(stdout.contains("p99_us="), "{stdout}");
    assert!(stdout.contains("status=pass"), "{stdout}");
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/lumen/tests/lumen_bench_cli.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/lumen/tests/lumen_bench_cli.rs` captured for the sorted_page_deep benchmark CLI smoke test.
```
