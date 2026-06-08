---
number: 569
title: "Add soft keywords interaction tests (match, case, type)"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #569 — Add soft keywords interaction tests (match, case, type)

## Context
`match`, `case`, and `type` are soft keywords — they are valid identifiers except in specific syntactic contexts. This is a major parser challenge.

## Test cases
```python
# match/case as regular identifiers
match = 1
case = 2
type = int
match.case = 3
print(match, case, type)

# match statement vs match variable
match match:
    case case:
        pass

# type as identifier vs type statement
type = int
type Point = tuple[float, float]

# In function signatures
def match(case, type):
    return case

# As class names
class match: pass
class case: pass

# In comprehensions
[match for match in range(10)]
{case: match for case, match in items}

# As import names
# import match  # if module exists
# from case import type

# Complex interactions
match (match := case):
    case [*type]:
        pass
```

## Task
Create `tests/fixtures/parse/edge_cases/soft_keywords.py` with `# RUN: parse`.
