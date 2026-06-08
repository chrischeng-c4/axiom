---
number: 759
title: "Py3.12 conformance: Data structure ops — list, dict, set, tuple, str, bytes"
state: open
labels: [enhancement, P1, crate:mamba]
group: "py312-conformance-p1"
---

# #759 — Py3.12 conformance: Data structure ops — list, dict, set, tuple, str, bytes

## Parent

Part of #750

## Goal

Verify all data structure operations match CPython 3.12 behavior.

## Scope

### list
- [ ] append, extend, insert, pop, remove, sort, reverse, copy, clear
- [ ] Slicing: `a[1:3]`, `a[::2]`, `a[::-1]`
- [ ] List comprehension edge cases
- [ ] `__contains__`, `__eq__`, `__lt__` (lexicographic)

### dict
- [ ] get, setdefault, update, pop, popitem, keys/values/items
- [ ] `dict | dict` merge (PEP 584, Py3.9+)
- [ ] Dict comprehension, dict ordering (insertion order)
- [ ] `__missing__` for subclasses

### set
- [ ] add, discard, remove, pop, clear
- [ ] union, intersection, difference, symmetric_difference
- [ ] Set comprehension, frozenset

### tuple
- [ ] Immutability, unpacking, `*` unpacking
- [ ] Hashing, comparison (lexicographic)
- [ ] Named tuples interop

### str
- [ ] All 47+ methods: split, join, strip, replace, find, format, encode, etc.
- [ ] f-string edge cases, format spec mini-language
- [ ] Unicode normalization

### bytes / bytearray
- [ ] decode, hex, fromhex, slice
- [ ] bytes literals, bytearray mutability

## Current State

Basic ops work. Edge cases and CPython-matching semantics need verification.
