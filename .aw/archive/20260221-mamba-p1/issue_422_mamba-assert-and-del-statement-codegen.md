---
number: 422
title: "mamba: assert and del statement codegen"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #422 — mamba: assert and del statement codegen

## Description

`assert` and `del` statements are parsed but may not be fully wired through HIR→MIR→codegen.

## Requirements

### assert
- R1: `assert condition` — raise AssertionError if falsy
- R2: `assert condition, message` — with custom message
- R3: Optimizable: skip asserts when `-O` flag is set (future)

### del
- R4: `del x` — delete local variable (unbind name)
- R5: `del obj.attr` — delete attribute (calls `__delattr__`)
- R6: `del lst[i]` — delete item (calls `__delitem__`)
- R7: `del d[key]` — delete dict entry

## Priority

P1 — `assert` is used in every test and many programs; `del` is common for dict/list manipulation.
