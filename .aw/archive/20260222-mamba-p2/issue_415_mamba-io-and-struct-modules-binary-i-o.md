---
number: 415
title: "mamba: io and struct modules (binary I/O)"
state: open
labels: [enhancement, P2, crate:mamba]
dependencies: [405]
---

# #415 — mamba: io and struct modules (binary I/O)

## Description

Implement `io` and `struct` modules for binary I/O operations.

## Requirements

### io module
- R1: `io.StringIO` — in-memory text stream
- R2: `io.BytesIO` — in-memory binary stream
- R3: `io.BufferedReader` / `io.BufferedWriter`
- R4: `io.TextIOWrapper`

### struct module
- R5: `struct.pack(fmt, *values)` — pack values into bytes
- R6: `struct.unpack(fmt, buffer)` — unpack bytes into values
- R7: `struct.calcsize(fmt)` — size of packed format
- R8: Format strings: `>`, `<`, `!`, `@` byte order + `b`, `h`, `i`, `q`, `f`, `d`, etc.

## Dependencies

Depends on #405 (bytes/bytearray type).

## Priority

P2 — needed for binary file formats, network protocols.
