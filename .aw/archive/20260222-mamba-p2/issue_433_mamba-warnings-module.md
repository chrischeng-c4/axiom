---
number: 433
title: "mamba: warnings module"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #433 — mamba: warnings module

## Description

Implement `warnings` module for issuing and filtering warnings.

## Requirements

- R1: `warnings.warn(message, category=UserWarning, stacklevel=1)`
- R2: Warning categories: `UserWarning`, `DeprecationWarning`, `FutureWarning`, `SyntaxWarning`, `RuntimeWarning`
- R3: `warnings.filterwarnings(action, message, category, module, lineno)`
- R4: `warnings.simplefilter(action)` — shortcut for simple filters
- R5: Actions: `"error"`, `"ignore"`, `"always"`, `"default"`, `"once"`
- R6: `@warnings.deprecated` decorator (Python 3.13 but useful)

## Priority

P2 — used by many libraries for deprecation notices.
