---
number: 440
title: "mamba: base64 module"
state: open
labels: [enhancement, P2, crate:mamba]
dependencies: [405]
---

# #440 — mamba: base64 module

## Description

Implement `base64` module for binary-to-text encoding.

## Requirements

- R1: `base64.b64encode(data)` — encode bytes to base64
- R2: `base64.b64decode(data)` — decode base64 to bytes
- R3: `base64.urlsafe_b64encode(data)` / `urlsafe_b64decode(data)`
- R4: `base64.b32encode(data)` / `b32decode(data)`
- R5: `base64.b16encode(data)` / `b16decode(data)` (hex)

## Dependencies

Depends on #405 (bytes type).

## Priority

P2 — commonly used for encoding tokens, API keys, binary data.
