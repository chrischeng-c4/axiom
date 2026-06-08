---
number: 568
title: "Add parenthesized context managers fixture (PEP 617)"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #568 — Add parenthesized context managers fixture (PEP 617)

## Context
PEP 617 (Python 3.10) enabled parenthesized context managers in `with` statements via the new PEG parser.

## Test cases
```python
# Parenthesized with statement
with (open('a') as f):
    pass

# Multiple managers
with (open('a') as f, open('b') as g):
    pass

# Trailing comma
with (open('a') as f, open('b') as g,):
    pass

# Multi-line
with (
    open('a') as f,
    open('b') as g,
    open('c') as h,
):
    pass

# Without as
with (open('a'), open('b')):
    pass

# Nested parenthesized
with (open('a') as f):
    with (open('b') as g):
        pass
```

## Task
Create `tests/fixtures/parse/edge_cases/parenthesized_context_managers.py` with `# RUN: parse`.
