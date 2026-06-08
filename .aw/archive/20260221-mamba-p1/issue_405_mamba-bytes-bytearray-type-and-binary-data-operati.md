---
number: 405
title: "mamba: bytes/bytearray type and binary data operations"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #405 — mamba: bytes/bytearray type and binary data operations

## Description

Implement `bytes` and `bytearray` types for binary data handling. This is a fundamental Python type needed for network I/O, file I/O (binary mode), encoding/decoding, and many stdlib modules.

## Requirements

### bytes (immutable)
- `b"hello"` literal syntax
- Indexing returns int, slicing returns bytes
- Methods: `decode`, `hex`, `find`, `replace`, `split`, `join`, `strip`, `startswith`, `endswith`, `count`, `index`, `upper`, `lower`
- Operators: `+` (concat), `*` (repeat), `in` (membership)
- `bytes()`, `bytes(n)`, `bytes(iterable)` constructors

### bytearray (mutable)
- Same methods as bytes plus mutating operations
- `append`, `extend`, `insert`, `pop`, `remove`, `reverse`, `clear`
- Buffer protocol support

### Integration
- `str.encode()` → bytes
- `bytes.decode()` → str
- File I/O binary mode (`open("f", "rb")`)

## Priority

P1 — fundamental type required by networking, file I/O, and many stdlib modules.
