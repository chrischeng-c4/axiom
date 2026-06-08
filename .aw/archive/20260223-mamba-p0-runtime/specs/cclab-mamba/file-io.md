---
id: file-io
type: spec
title: "File I/O Runtime"
version: 1
spec_type: utility
spec_group: cclab-mamba
created_at: 2026-02-15T17:32:02.740697+00:00
updated_at: 2026-02-15T17:32:02.740697+00:00
requirements:
  total: 6
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
depends:
  - method-dispatch
  - exception-hierarchy
changes:
  - file: crates/mamba/src/runtime/rc.rs
    action: MODIFY
    description: "Add ObjData::File variant"
  - file: crates/mamba/src/runtime/file_io.rs
    action: CREATE
    description: "New file I/O module"
  - file: crates/mamba/src/runtime/mod.rs
    action: MODIFY
    description: "Add pub mod file_io"
  - file: crates/mamba/src/runtime/symbols.rs
    action: MODIFY
    description: "Register open builtin and file methods"
history:
  - timestamp: 2026-02-15T17:32:02.740697+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# File I/O Runtime

## Overview

Implements file I/O as a new ObjData::File variant wrapping a Rust File handle. Provides open(), read(), write(), close(), readline(), readlines(), writelines() as extern C runtime functions. The open() builtin returns a file object. File methods are wired into the method-dispatch table. Errors raise FileNotFoundError or IOError from the exception hierarchy.

## Requirements

### R1 - ObjData::File variant

```yaml
id: R1
priority: high
status: draft
```

Add File variant to ObjData enum containing: path (String), handle (Option of File), mode (String). Handle is Option to support closed files.

### R2 - Open builtin

```yaml
id: R2
priority: high
status: draft
```

mb_open(path, mode?) -> File object. Modes: 'r' (default), 'w', 'a', 'rb', 'wb'. Raises FileNotFoundError if file doesn't exist in read mode.

### R3 - File read methods

```yaml
id: R3
priority: high
status: draft
```

mb_file_read(self) -> string contents. mb_file_readline(self) -> next line. mb_file_readlines(self) -> list of lines.

### R4 - File write methods

```yaml
id: R4
priority: high
status: draft
```

mb_file_write(self, data) -> bytes written. mb_file_writelines(self, lines) -> None.

### R5 - File close and lifecycle

```yaml
id: R5
priority: high
status: draft
```

mb_file_close(self) -> None. Sets handle to None. Subsequent operations on closed file raise ValueError.

### R6 - Symbol registration

```yaml
id: R6
priority: high
status: draft
```

Register open as builtin, file methods in dispatch table under ObjData::File. Add pub mod file_io to runtime/mod.rs.

## Acceptance Criteria

### Scenario: Read file

- **WHEN** f = open('test.txt'); data = f.read(); f.close()
- **THEN** data contains file contents

### Scenario: Write file

- **WHEN** f = open('out.txt', 'w'); f.write('hello'); f.close()
- **THEN** File out.txt contains 'hello'

### Scenario: FileNotFoundError

- **WHEN** open('nonexistent.txt')
- **THEN** Raises FileNotFoundError

### Scenario: Read after close

- **WHEN** f = open('test.txt'); f.close(); f.read()
- **THEN** Raises ValueError: I/O operation on closed file

</spec>
