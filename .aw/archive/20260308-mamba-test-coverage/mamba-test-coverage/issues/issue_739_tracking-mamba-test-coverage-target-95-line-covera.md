---
number: 739
title: "[Tracking] Mamba test coverage — target 95%+ line coverage"
state: open
labels: [enhancement, type:tracking, crate:mamba]
group: "parser-lexer-coverage"
---

# #739 — [Tracking] Mamba test coverage — target 95%+ line coverage

## Coverage Standards (Language Project)

| Subsystem | Line Coverage Target | Priority |
|-----------|---------------------|----------|
| Core Runtime / Interpreter | 95–98% | P0 |
| JIT / AOT Codegen | 95–98% | P0 |
| Parser + Lexer | 95–98% | P0 |
| Type checker | 95–98% | P0 |
| HIR / MIR / Lowering | 95–98% | P0 |
| Name resolution | 95–98% | P0 |
| Native stdlib (common, 30+ modules) | 90–95% | P1 |
| Native stdlib (edge, 40+ modules) | 80%+ | P2 |
| FFI / Binding Layer | 100% | P0 |

## Current Baseline

- Overall: **30.07%** line coverage (9,342 / 31,067 coverable lines)
- 733 tests total
- Measured via `cargo tarpaulin`

## Measurement

- **Line coverage**: `cargo tarpaulin -p mamba --skip-clean`
- **Test distribution**: `/cclab:mamba:test-coverage` skill

## Sub-issues

- #740 Runtime core → 95–98%
- #741 Type checker → 95–98%
- #742 Stdlib modules → 80–95% (common 90–95%, edge 80%+)
- #743 HIR/MIR/Lowering → 95–98%
- #744 Codegen (JIT/AOT/LLVM) → 95–98%
- #745 Parser → 95–98%
- #746 Lexer → 95–98%
- #747 Name resolution → 95–98%
- #748 FFI → 100%
