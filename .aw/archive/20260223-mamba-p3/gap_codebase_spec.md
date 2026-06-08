---
change_id: mamba-p3
type: gap_codebase_spec
created_at: 2026-02-23T01:12:33.961654+00:00
updated_at: 2026-02-23T01:12:33.961654+00:00
---

# Gap Analysis: Codebase vs Spec

## Code Without Spec (medium severity)
1. **25+ P2 stdlib modules unspecced** — re, datetime, collections, itertools, functools, pathlib, random, shutil, tempfile, glob, io, struct, hashlib, base64, decimal, fractions, contextlib, copy, operator, weakref, traceback, warnings, inspect, enum, dataclasses all implemented but only mamba-stdlib-core covers sys/os/math/json.
2. **bytes/bytearray (#405)** — ObjData::Bytes/ByteArray in rc.rs, implemented in P1, no main spec.
3. **metaclasses/ABC (#407)** — Implemented in P1, no main spec.
4. **ExceptionGroup/except*** — exception.rs has full support, no spec.

## Spec Without Code (high severity)
1. **16 new P3 stdlib modules** — subprocess, csv, argparse, logging, typing, threading, socket/http, unittest, pickle, sqlite3, compression (gzip/zipfile/tarfile), pprint, textwrap, xml/html, array, string. Need both specs and implementation.
2. **Complex number full ops (#453)** — ObjData::Complex variant not added, cmath module not created. AST ComplexLit exists but runtime support is stub.
3. **eval/exec (#441)** — No parser re-entry mechanism. Architecturally complex.

## Summary
Main gap: 18 P3 issues need new specs and implementations. P1/P2 code has medium-severity spec gaps but is functional. All P3 DAG dependencies on closed issues are resolved.