---
number: 428
title: "mamba: shutil module (high-level file operations)"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #428 — mamba: shutil module (high-level file operations)

## Description

Implement `shutil` module for high-level file and directory operations.

## Requirements

- R1: `shutil.copy(src, dst)` — copy file
- R2: `shutil.copy2(src, dst)` — copy file preserving metadata
- R3: `shutil.copytree(src, dst)` — recursive directory copy
- R4: `shutil.rmtree(path)` — recursive directory removal
- R5: `shutil.move(src, dst)` — move file or directory
- R6: `shutil.which(name)` — find executable in PATH
- R7: `shutil.disk_usage(path)` — disk usage statistics
- R8: `shutil.make_archive()` / `shutil.unpack_archive()` (lower priority)

## Priority

P2 — commonly used for file management scripts.
