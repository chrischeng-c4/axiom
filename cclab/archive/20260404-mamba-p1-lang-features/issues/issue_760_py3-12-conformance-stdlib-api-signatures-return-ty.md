---
number: 760
title: "Py3.12 conformance: Stdlib API signatures & return types"
state: open
labels: [type:enhancement, priority:p1, crate:mamba]
group: "stdlib-conformance"
---

# #760 — Py3.12 conformance: Stdlib API signatures & return types

## Parent

Part of #750

## Goal

Verify implemented stdlib modules match CPython 3.12 API signatures and return types.

## Scope

- [ ] High-priority modules: os, sys, json, re, collections, itertools, functools
- [ ] Medium-priority: math, random, datetime, pathlib, io, hashlib
- [ ] API surface check: function signatures, default args, return types
- [ ] Edge cases: empty inputs, boundary values, error conditions
- [ ] Error types: verify correct exception raised for invalid inputs

## Current State

Many stdlib modules have native Rust implementations. API surface not systematically verified against CPython 3.12.
