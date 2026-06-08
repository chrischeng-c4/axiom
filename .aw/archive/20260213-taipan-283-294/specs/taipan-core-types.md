---
id: taipan-core-types
type: spec
title: "Core Data Structures (String, List, Dict, Tuple)"
version: 1
spec_type: data-model
tags: [data]
created_at: 2026-02-13T04:18:41.823113+00:00
updated_at: 2026-02-13T04:18:41.823113+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: true
  has_semantic_diagrams: false
  api_spec_type: json-schema
  diagrams:
    - type: class
      title: "Core Types Class Diagram"
history:
  - timestamp: 2026-02-13T04:18:41.823113+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Core Data Structures (String, List, Dict, Tuple)

## Overview

Implementation of Python's core built-in types: String, List, Dict, and Tuple. These types must match CPython's behavior, including method availability, mutability characteristics, and performance characteristics (e.g. O(1) dict lookups).

## Requirements

### R1 - String Implementation

```yaml
id: R1
priority: medium
status: draft
```

Implement string type supporting unicode, immutability, and standard operations (concatenation, slicing, indexing).

### R2 - List Implementation

```yaml
id: R2
priority: medium
status: draft
```

Implement list type supporting dynamic resizing, mutability, and standard operations (append, pop, insert, extend).

### R3 - Dict Implementation

```yaml
id: R3
priority: medium
status: draft
```

Implement dictionary type supporting key-value mapping, insertion order preservation, and standard operations.

### R4 - Tuple Implementation

```yaml
id: R4
priority: medium
status: draft
```

Implement tuple type supporting immutable sequences and hashing (if elements are hashable).

## Acceptance Criteria

### Scenario: String Concatenation

- **GIVEN** Two strings 'hello' and 'world'
- **WHEN** They are concatenated
- **THEN** Result should be 'helloworld'

### Scenario: List Append

- **GIVEN** An empty list
- **WHEN** Value 1 is appended
- **THEN** List should contain [1]

### Scenario: Dict Set/Get

- **GIVEN** An empty dict
- **WHEN** 'key' is set to 'value'
- **THEN** Retrieving 'key' should return 'value'

### Scenario: Tuple Immutability

- **GIVEN** A tuple (1, 2)
- **WHEN** Element 0 is assigned a new value
- **THEN** A TypeError should be raised

## Diagrams

### Core Types Class Diagram

```mermaid
classDiagram
    class PyString {
        +Vec<u8> data
        +concat(String other) String
        +slice(usize start, usize end) String
    }
    class PyList {
        +Vec<PyObject> elements
        +append(PyObject item) void
        +pop() PyObject
    }
    class PyDict {
        +IndexMap<PyObject, PyObject> entries
        +get(PyObject key) Option<PyObject>
        +set(PyObject key, PyObject value) void
    }
    class PyTuple {
        +Vec<PyObject> elements
        +get(usize index) PyObject
    }
```

## API Specification (JSON Schema)

```yaml
definitions:
  PyDict:
    properties:
      entries:
        additionalProperties: {}
        type: object
    type: object
  PyList:
    properties:
      elements:
        items: {}
        type: array
    type: object
  PyString:
    properties:
      value:
        type: string
    type: object
  PyTuple:
    properties:
      elements:
        items: {}
        type: array
    type: object
```

</spec>
