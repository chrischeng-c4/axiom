---
id: mamba-stdlib-binary
type: spec
title: "Stdlib: io/struct, hashlib, base64"
version: 1
spec_type: utility
created_at: 2026-02-22T11:21:14.934133+00:00
updated_at: 2026-02-22T11:21:14.934133+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-22T11:21:14.934133+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Stdlib: io/struct, hashlib, base64

## Overview

Implement four binary-data stdlib modules for Mamba: io (StringIO/BytesIO in-memory streams), struct (binary packing/unpacking), hashlib (cryptographic hashing: md5, sha256, sha512), and base64 (base64/base32/base16 encoding/decoding). All depend on bytes type (#405) being available.

## Requirements

### R1 - io module

```yaml
id: R1
priority: high
status: draft
```

Create io_mod.rs. StringIO: mb_stringio_new, mb_stringio_write, mb_stringio_read, mb_stringio_getvalue (backed by String buffer). BytesIO: mb_bytesio_new, mb_bytesio_write, mb_bytesio_read, mb_bytesio_getvalue (backed by Vec<u8>).

### R2 - struct module

```yaml
id: R2
priority: medium
status: draft
```

Create struct_mod.rs. mb_struct_pack(fmt, args) packs values into bytes. mb_struct_unpack(fmt, data) unpacks bytes into tuple. mb_struct_calcsize(fmt) returns format size. Support basic format chars: b/B (i8/u8), h/H (i16/u16), i/I (i32/u32), q/Q (i64/u64), f/d (f32/f64).

### R3 - hashlib module

```yaml
id: R3
priority: medium
status: draft
```

Create hashlib_mod.rs. Hash constructors: mb_hashlib_md5, mb_hashlib_sha256, mb_hashlib_sha512 return hash dict. Methods: update(data) appends bytes, hexdigest() returns hex string. Implement via simple hash algorithms or store accumulated data for final digest.

### R4 - base64 module

```yaml
id: R4
priority: medium
status: draft
```

Create base64_mod.rs. mb_base64_b64encode(data) and mb_base64_b64decode(data). mb_base64_urlsafe_b64encode/decode. Implement base64 encoding table directly (no external crate).

## Acceptance Criteria

### Scenario: StringIO write and read

- **WHEN** sio = StringIO(); sio.write('hello'); sio.getvalue()
- **THEN** Returns 'hello'

### Scenario: base64 roundtrip

- **WHEN** base64.b64decode(base64.b64encode(b'hello'))
- **THEN** Returns b'hello'

### Scenario: struct pack/unpack

- **WHEN** struct.unpack('ii', struct.pack('ii', 1, 2))
- **THEN** Returns (1, 2)

</spec>
