---
change: mamba-all-p1
group: exception-handling
date: 2026-03-19
status: answered
---

# Pre-Clarifications

### Q1: General
- **Answer**: FULLY IMPLEMENTED (PEP 654). Parser supports except* in stmt_compound.rs:289-295 with is_star flag. HIR propagates is_star in HirExceptHandler. Runtime has ExceptionGroup (exception.rs:417-456) and mb_except_star() (458-528) for splitting by type. Test fixtures exist for parse and conformance.

### Q2: General
- **Answer**: FULLY IMPLEMENTED with chaining. MbException struct (exception.rs:10-24) has: exc_type, message, cause (Option<Box<MbException>>), context (Option<Box<MbException>>), suppress_context (bool), traceback (Vec<(String, u32, String)>). Helper methods with_cause/with_context at lines 38-46. mb_raise_from() for explicit chaining, mb_raise_with_context() for implicit. Stored as MbValue with __cause__/__context__/__suppress_context__ fields.

### Q3: General
- **Answer**: EXISTS in exception.rs with chained exception display. Traceback rendering shows file/line/function stack. Chaining display shows 'The above exception was the direct cause...' and 'During handling of the above exception...' messages. Could benefit from integration with the diagnostic module for richer ANSI colored output and code snippets.

