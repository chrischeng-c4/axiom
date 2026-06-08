---
number: 424
title: "mamba: os.path and extended os module"
state: open
labels: [enhancement, P1, crate:mamba]
---

# #424 — mamba: os.path and extended os module

## Description

Extend the existing `os` module with `os.path` and commonly used os functions.

## Requirements

### os.path
- R1: `os.path.join(*paths)` — join path components
- R2: `os.path.exists(path)`, `os.path.isfile(path)`, `os.path.isdir(path)`
- R3: `os.path.basename(path)`, `os.path.dirname(path)`
- R4: `os.path.abspath(path)`, `os.path.realpath(path)`
- R5: `os.path.splitext(path)` — split extension
- R6: `os.path.split(path)` — split head/tail
- R7: `os.path.expanduser(path)` — expand ~
- R8: `os.path.getsize(path)` — file size

### os functions
- R9: `os.listdir(path=".")` — list directory contents
- R10: `os.mkdir(path)`, `os.makedirs(path, exist_ok=False)`
- R11: `os.remove(path)`, `os.rmdir(path)`
- R12: `os.rename(src, dst)`
- R13: `os.walk(top)` — recursive directory walk
- R14: `os.path.sep`, `os.sep`, `os.linesep` constants
- R15: `os.stat(path)` — file metadata

## Current State

Basic `os.environ`, `os.getcwd` already exist.

## Priority

P1 — file path manipulation is needed by almost every program that does file I/O.
