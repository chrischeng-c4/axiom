---
number: 567
title: "Add PEP 701 relaxed f-string grammar tests (Python 3.12 specific)"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #567 — Add PEP 701 relaxed f-string grammar tests (Python 3.12 specific)

## Context
PEP 701 (Python 3.12) fundamentally changes f-string parsing — f-strings are now parsed by the regular parser instead of a separate tokenizer. This enables:
- Arbitrary nesting depth
- Backslashes in f-string expressions
- Comments in multi-line f-string expressions
- Reuse of same quote type

## Test cases
```python
# Arbitrary nesting
f"{f"{f"{x}"}"}"

# Same quote reuse (NEW in 3.12)
f"{'hello'}"
f"{"world"}"

# Backslash in expression (NEW in 3.12)
f"{'\n'.join(items)}"
f"{chr(92)}"

# Multi-line with comments
f"{
    x  # this is a comment
    + y
}"

# Nested format specs
f"{value:{width}.{precision}}"

# Lambda in f-string
f"{(lambda x: x + 1)(5)}"

# Dict with same quotes
f"{{'key': 'value'}}"
```

## Task
Create `tests/fixtures/parse/edge_cases/pep701_fstring.py` with `# RUN: parse`.
