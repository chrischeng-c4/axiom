---
number: 452
title: "mamba: string module (constants and Template)"
state: open
labels: [enhancement, crate:mamba, P3]
---

# #452 — mamba: string module (constants and Template)

## Description

Implement `string` module with constants and Template class.

## Requirements

- R1: Constants: `string.ascii_lowercase`, `ascii_uppercase`, `ascii_letters`, `digits`, `hexdigits`, `octdigits`, `punctuation`, `whitespace`, `printable`
- R2: `string.Template(template)` — simple string substitution
- R3: `template.substitute(mapping)` / `safe_substitute(mapping)`
- R4: `string.Formatter` class (lower priority)
- R5: `string.capwords(s)` — capitalize words

## Priority

P3 — string constants are occasionally useful; Template rarely used in modern code.
