---
change: mamba-all-p1
group: repl
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: No true incremental compilation. Each REPL iteration creates fresh CraneliftJitBackend::new() (repl.rs:178) and passes entire MIR module. However, accumulated state (variables, functions, classes) persists across iterations via lowering state (repl.rs:153-193). Full REPL already exists at driver/repl.rs (383 lines) with multi-line input, expression evaluation, magic commands, error recovery.

### Q2: General
- **Answer**: IMPLEMENTED. needs_continuation() function in repl.rs:229-272 detects incomplete statements by checking for unclosed brackets, trailing colons, and indentation levels. Uses both parsing attempt and heuristic detection.

### Q3: General
- **Answer**: No rustyline used currently. REPL reads from stdin directly (repl.rs:62). Adding rustyline as an optional feature flag is recommended for readline editing, history, and tab completion without making it a mandatory dependency.

