---
number: 1035
title: "test(mamba): per-module test coverage gaps — lower, resolve, runtime severely undertested"
state: open
labels: [type:enhancement, priority:p1, crate:mamba, type:test]
group: "test-coverage-gaps"
---

# #1035 — test(mamba): per-module test coverage gaps — lower, resolve, runtime severely undertested

## Current Coverage (measured by cargo-llvm-cov)

```
Line coverage:   74% (7,727 lines uncovered out of 29,713)
Branch coverage: 86% (771 branches uncovered out of 5,436)
Target:          100% line + 100% branch
```

## Files < 50% Line Coverage (29 files)

### stdlib (22 files — biggest gap)
| File | Coverage |
|------|----------|
| queue_mod.rs | 4% |
| statistics_mod.rs | 5% |
| shlex_mod.rs | 7% |
| calendar_mod.rs | 8% |
| locale_mod.rs | 10% |
| lzma_mod.rs | 11% |
| zlib_mod.rs | 11% |
| secrets_mod.rs | 12% |
| bisect_mod.rs | 14% |
| abc_mod.rs | 14% |
| uuid_mod.rs | 15% |
| numbers_mod.rs | 9% |
| argparse_mod.rs | 20% |
| platform_mod.rs | 25% |
| unittest_mod.rs | 31% |
| socket_mod.rs | 34% |
| array_mod.rs | 35% |
| errno_mod.rs | 37% |
| traceback_mod.rs | 37% |
| codecs_mod.rs | 46% |
| logging_mod.rs | 46% |
| pickle_mod.rs | 47% |

### Core modules
| File | Coverage |
|------|----------|
| ffi/c_types.rs | **0%** |
| driver/mod.rs | 33% |
| codegen/cranelift/mod.rs | 45% |
| runtime/file_io.rs | 50% |
| runtime/stdlib/threading_mod.rs | 49% |
| runtime/stdlib/sqlite3_mod.rs | 49% |

## Files 50-75% (need to reach 100%)

| File | Coverage |
|------|----------|
| types/check_expr.rs | 66% |
| codegen/cranelift/aot.rs | 67% |
| codegen/cranelift/jit.rs | 69% |
| lexer/token.rs | 70% |
| lower/ast_to_hir.rs | 75% |
| lower/hir_to_mir.rs | 78% |
| driver/module_graph.rs | 76% |
| parser/expr_compound.rs | 78% |

## Files > 90% (close to done)
value.rs (98%), symbols.rs (100%), context.rs (100%), ty.rs (100%), generic.rs (95%), protocol.rs (98%), ffi/safety.rs (100%), string_ops.rs (86%), builtins.rs (87%)

## Approach

Work from lowest coverage up. Each batch: pick 5-10 worst files, add inline tests until coverage reaches 90%+, measure again.
