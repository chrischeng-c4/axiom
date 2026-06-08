---
change: mamba-p1-lang-features
group: exception-chaining
date: 2026-04-04
---

# Requirements

Exception chaining: add __cause__, __context__, suppress_context to MbException. Implement 'raise X from Y' syntax. Update exception display to show chain. Changes span parser (raise-from), runtime (exception.rs), and codegen.
