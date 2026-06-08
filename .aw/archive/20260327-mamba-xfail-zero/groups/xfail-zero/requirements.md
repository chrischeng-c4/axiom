---
change: mamba-xfail-zero
group: xfail-zero
date: 2026-03-27
---

# Requirements

Eliminate all 34 remaining xfail conformance tests. Fix runtime, parser, and codegen so every test matches CPython 3.12 output exactly. Categories: 15 stdlib module tests (collections, csv, datetime, functools, hashlib, io, itertools, json, math, random, re, struct), 4 data_structures (bytes/dict/list/set edge cases — need KeyError/IndexError/ValueError raising), 4 language (comprehension scope, decorators, lambda, pattern matching edge cases), 3 iterators (callable sentinel iter(f,s), composition, custom __iter__/__next__, unpacking), 2 generators (state attributes gi_yieldfrom/gi_running, yield from passthrough), 1 class_system (MRO edge cases), 1 exceptions (chaining __cause__/__context__).
