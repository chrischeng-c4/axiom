---
number: 445
title: "mamba: gzip, zipfile, tarfile modules (compression)"
state: open
labels: [enhancement, crate:mamba, P3]
dependencies: [405]
---

# #445 — mamba: gzip, zipfile, tarfile modules (compression)

## Description

Implement compression/archive modules.

## Requirements

### gzip
- R1: `gzip.open(filename, mode)` — open gzipped file
- R2: `gzip.compress(data)` / `gzip.decompress(data)` — in-memory

### zipfile
- R3: `zipfile.ZipFile(file, mode)` — open zip archive
- R4: `.read(name)`, `.extractall(path)`, `.namelist()`
- R5: `.write(filename)`, `.writestr(name, data)`

### tarfile
- R6: `tarfile.open(name, mode)` — open tar archive
- R7: `.extractall(path)`, `.getmembers()`, `.add(name)`

## Dependencies

Depends on #405 (bytes type).

## Priority

P3 — needed for working with archives and compressed data.
