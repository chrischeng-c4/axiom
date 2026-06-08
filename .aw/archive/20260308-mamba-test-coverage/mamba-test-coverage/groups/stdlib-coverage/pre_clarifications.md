---
change: mamba-test-coverage
group: stdlib-coverage
date: 2026-03-08
status: answered
---

# Pre-Clarifications

### Q1: stdlib-common-vs-edge
- **Answer**: Common (90-95% target): sys, os, math, json, re, collections, datetime, pathlib, io, hashlib, base64, random, functools, itertools, copy, operator, struct, csv, logging, string_constants, time, decimal, fractions, contextlib, traceback, inspect, enum, dataclasses. Edge (80% target): all remaining modules (configparser, difflib, hmac, heapq, bisect, uuid, calendar, statistics, numbers, unicodedata, zlib, bz2, lzma, queue, signal, secrets, asyncio, platform, shlex, locale, abc, etc.).

### Q2: stdlib-behavioral-verification
- **Answer**: For now, test internal logic correctness with hardcoded expected values derived from CPython 3.12 behavior. E.g., math.sqrt(4) should return 2.0, json.dumps({}) should return '{}'. This provides behavioral verification without needing a CPython conformance runner. Document the CPython version used as reference in test comments.

