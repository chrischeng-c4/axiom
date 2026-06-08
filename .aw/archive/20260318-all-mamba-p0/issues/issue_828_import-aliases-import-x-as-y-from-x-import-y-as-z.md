---
number: 828
title: "Import aliases — import X as Y, from X import Y as Z"
state: open
labels: [enhancement, P0, crate:mamba]
group: "module-system"
---

# #828 — Import aliases — import X as Y, from X import Y as Z

## Summary

Support import aliasing syntax:
- `import numpy as np`
- `from collections import OrderedDict as OD`
- `from typing import List as L`

This is fundamental Python syntax and is used in virtually every real-world Python file.

## Current State

Import aliases are listed in `cpython_known_failures.toml` as unsupported. The parser does not handle the `as` clause in import statements.

## Scope

- **Parser**: Handle `as` in `import` and `from...import` statements
- **Resolver**: Bind aliased name in scope
- **Type checker**: Propagate type through alias
- **Codegen**: Alias is just a name binding, minimal codegen change
