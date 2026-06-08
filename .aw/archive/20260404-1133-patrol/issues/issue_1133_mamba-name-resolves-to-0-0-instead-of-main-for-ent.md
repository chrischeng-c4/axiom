---
number: 1133
title: "mamba: __name__ resolves to 0.0 instead of \"__main__\" for entry point"
state: open
labels: [type:bug, priority:p1, crate:mamba]
group: "fix-name-builtin"
---

# #1133 — mamba: __name__ resolves to 0.0 instead of "__main__" for entry point

## Problem

When running a script via `cclab mamba run`, the `__name__` builtin resolves to `0.0` (float) instead of `"__main__"` (string).

## Reproduction

```python
print("__name__:", __name__)
if __name__ == "__main__":
    print("running as main")
```

```bash
$ cclab mamba run test.py
__name__: 0.0
```

The `if __name__ == "__main__"` guard never enters, so scripts with the standard Python entry point pattern silently do nothing.

## Expected

`__name__` should be `"__main__"` when the script is the entry point.

## Impact

Conductor's `main.py` uses the standard `if __name__ == "__main__": main()` pattern — this bug prevents it from starting.
