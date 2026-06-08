---
number: 1112
title: "fix(mamba): try/except block with dict/set literal triggers parse error"
state: open
labels: [type:bug, priority:p1, crate:mamba]
group: "try-except-dict-set-parse"
---

# #1112 — fix(mamba): try/except block with dict/set literal triggers parse error

## Problem

```python
try:
    d = {}['x']       # parse error
except KeyError:
    print('caught')

try:
    s = {1,2}.remove(99)  # parse error  
except KeyError:
    print('caught')
```

Dict and set literals inside try blocks trigger a parse error, likely due to `{` being ambiguous between block start and literal.

## Impact

~2 conformance xfail fixtures blocked.

## Affected Files

- `crates/mamba/src/parser/` — expression parsing in try block context
