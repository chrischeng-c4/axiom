---
id: mamba-stdlib-re
type: spec
title: "Stdlib: re (regular expressions)"
version: 1
spec_type: utility
created_at: 2026-02-22T11:20:11.083197+00:00
updated_at: 2026-02-22T11:20:11.083197+00:00
requirements:
  total: 7
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
    - R7
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-22T11:20:11.083197+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Stdlib: re (regular expressions)

## Overview

Implement the re (regular expressions) stdlib module for Mamba runtime. Provides re.compile, re.search, re.match, re.findall, re.sub, re.split, re.escape functions with Python-compatible API shape. Backed by simple pattern matching initially.

## Requirements

### R1 - re module registration

```yaml
id: R1
priority: high
status: draft
```

Create re_mod.rs with register() function. Register module 're' with attrs mapping function names to symbol strings.

### R2 - re.search(pattern, string)

```yaml
id: R2
priority: high
status: draft
```

mb_re_search: find first occurrence of pattern in string. Returns match dict {group: str, start: int, end: int} or None.

### R3 - re.match(pattern, string)

```yaml
id: R3
priority: high
status: draft
```

mb_re_match: match pattern at start of string only. Returns match dict or None.

### R4 - re.findall(pattern, string)

```yaml
id: R4
priority: high
status: draft
```

mb_re_findall: return list of all non-overlapping matches as strings.

### R5 - re.sub(pattern, repl, string)

```yaml
id: R5
priority: medium
status: draft
```

mb_re_sub: replace all occurrences of pattern with repl in string.

### R6 - re.split(pattern, string)

```yaml
id: R6
priority: medium
status: draft
```

mb_re_split: split string by pattern occurrences. Return list of strings.

### R7 - re.escape(string)

```yaml
id: R7
priority: low
status: draft
```

mb_re_escape: escape all special regex characters in string.

## Acceptance Criteria

### Scenario: findall extracts matches

- **WHEN** re.findall('\\d+', 'a1b22c333')
- **THEN** Returns ['1', '22', '333']

### Scenario: sub replaces pattern

- **WHEN** re.sub('world', 'Python', 'hello world')
- **THEN** Returns 'hello Python'

</spec>
