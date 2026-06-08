---
id: mamba-stdlib-fs
type: spec
title: "Stdlib: pathlib, shutil, tempfile, glob"
version: 1
spec_type: utility
created_at: 2026-02-22T11:20:51.704493+00:00
updated_at: 2026-02-22T11:20:51.704493+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-22T11:20:51.704493+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Stdlib: pathlib, shutil, tempfile, glob

## Overview

Implement four file-system related stdlib modules for Mamba: pathlib (object-oriented path manipulation with Path class), shutil (high-level file operations: copy, move, rmtree), tempfile (temporary file/directory creation), and glob (file pattern matching with wildcards).

## Requirements

### R1 - pathlib module registration

```yaml
id: R1
priority: high
status: draft
```

Create pathlib_mod.rs with register(). Functions: mb_pathlib_new, mb_pathlib_exists, mb_pathlib_is_file, mb_pathlib_is_dir, mb_pathlib_name, mb_pathlib_stem, mb_pathlib_suffix, mb_pathlib_parent, mb_pathlib_joinpath, mb_pathlib_read_text, mb_pathlib_write_text, mb_pathlib_mkdir, mb_pathlib_resolve.

### R2 - Path methods using std::path

```yaml
id: R2
priority: high
status: draft
```

Path is represented as a string MbValue. Methods extract components via std::path::Path. read_text/write_text use std::fs.

### R3 - shutil module

```yaml
id: R3
priority: medium
status: draft
```

Create shutil_mod.rs. Functions: mb_shutil_copy (file copy), mb_shutil_copytree (recursive dir copy), mb_shutil_rmtree (recursive delete), mb_shutil_move (rename/move), mb_shutil_which (find executable in PATH).

### R4 - tempfile module

```yaml
id: R4
priority: medium
status: draft
```

Create tempfile_mod.rs. Functions: mb_tempfile_mkstemp (create temp file, return path), mb_tempfile_mkdtemp (create temp dir), mb_tempfile_gettempdir (return system temp dir).

### R5 - glob module

```yaml
id: R5
priority: medium
status: draft
```

Create glob_mod.rs. Functions: mb_glob_glob(pattern) returns list of matching file paths. Implement simple * and ? wildcard matching using std::fs::read_dir recursion.

## Acceptance Criteria

### Scenario: pathlib extracts components

- **GIVEN** p = Path('/usr/local/bin/python')
- **WHEN** p.name, p.stem, p.suffix, p.parent
- **THEN** Returns 'python', 'python', '', '/usr/local/bin'

### Scenario: glob finds files

- **GIVEN** directory with files a.txt, b.txt, c.py
- **WHEN** glob.glob('*.txt')
- **THEN** Returns ['a.txt', 'b.txt']

</spec>
