---
number: 742
title: "Test coverage: Stdlib modules — 90–95% (common) / 80%+ (edge)"
state: open
labels: [enhancement, P1, crate:mamba]
group: "stdlib-coverage"
---

# #742 — Test coverage: Stdlib modules — 90–95% (common) / 80%+ (edge)

## Target
- **Common stdlib** (sys, os, math, json, re, collections, datetime, pathlib, etc.): **90–95%** line coverage
- **Edge stdlib** (calendar, locale, unicodedata, configparser, etc.): **80%+** line coverage

## Current
- 69/72 modules have tests, but 29 have only 1 test
- 3 modules with NO tests: dataclasses, enum, time

## Scope
All 72 modules in `src/runtime/stdlib/`

## Approach
1. Prioritize common modules first (higher usage = higher target)
2. Every module function must have at least one positive + one negative test
3. Edge cases: empty inputs, type errors, boundary values
