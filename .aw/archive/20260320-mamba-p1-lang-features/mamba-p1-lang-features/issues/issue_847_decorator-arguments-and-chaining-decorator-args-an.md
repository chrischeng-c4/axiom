---
number: 847
title: "Decorator arguments and chaining — @decorator(args) and stacked decorators"
state: open
labels: [enhancement, P1, crate:mamba]
group: "decorator-full"
---

# #847 — Decorator arguments and chaining — @decorator(args) and stacked decorators

## Summary

Ensure full decorator support including:
```python
@app.route("/api")           # attribute access in decorator
@cache(maxsize=128)          # decorator with arguments
@log
@validate
def handler(req):            # stacked decorators
    ...
```

## Scope

- **Parser**: Decorator expressions can be any expression (not just names)
- **Type checker**: Decorator return type replaces decorated function type
- **Codegen**: Apply decorators in reverse order (bottom-up)
