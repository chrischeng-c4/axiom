---
number: 416
title: "mamba: hashlib module"
state: open
labels: [enhancement, P2, crate:mamba]
dependencies: [405]
---

# #416 — mamba: hashlib module

## Description

Implement `hashlib` for cryptographic hash functions.

## Requirements

- R1: `hashlib.md5(data)`, `hashlib.sha1(data)`, `hashlib.sha256(data)`, `hashlib.sha512(data)`
- R2: `.hexdigest()`, `.digest()`, `.update(data)` methods
- R3: `hashlib.new(name, data)` constructor
- R4: `hashlib.algorithms_available` and `hashlib.algorithms_guaranteed`

## Dependencies

Depends on #405 (bytes type).

## Priority

P2 — commonly used for checksums, security.
