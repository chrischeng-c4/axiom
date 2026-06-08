---
id: mamba-stdlib-collections
type: spec
title: "Stdlib: collections (defaultdict, Counter, deque, OrderedDict)"
version: 1
spec_type: utility
created_at: 2026-02-22T11:20:31.703515+00:00
updated_at: 2026-02-22T11:20:31.703515+00:00
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
  - timestamp: 2026-02-22T11:20:31.703515+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Stdlib: collections (defaultdict, Counter, deque, OrderedDict)

## Overview

Implement the collections stdlib module for Mamba runtime. Provides defaultdict (dict with factory default), Counter (element counting), deque (double-ended queue backed by VecDeque), and OrderedDict (insertion-ordered dict, which is default dict behavior in Mamba).

## Requirements

### R1 - collections module registration

```yaml
id: R1
priority: high
status: draft
```

Create collections_mod.rs with register(). Register module 'collections' with attrs for defaultdict, Counter, deque, OrderedDict.

### R2 - defaultdict

```yaml
id: R2
priority: high
status: draft
```

mb_defaultdict_new(factory): create dict-like object that calls factory() for missing keys. Store factory as closure attribute on the dict.

### R3 - Counter

```yaml
id: R3
priority: high
status: draft
```

mb_counter_new(iterable): create dict of element→count. mb_counter_most_common(counter, n): return top-n pairs as list of tuples.

### R4 - deque

```yaml
id: R4
priority: high
status: draft
```

mb_deque_new(): create list-backed deque. mb_deque_append, mb_deque_appendleft, mb_deque_pop, mb_deque_popleft, mb_deque_len.

### R5 - OrderedDict

```yaml
id: R5
priority: low
status: draft
```

mb_ordereddict_new(): alias for dict (Mamba dicts are already insertion-ordered). Provided for API compatibility.

## Acceptance Criteria

### Scenario: Counter counts elements

- **WHEN** Counter(['a','b','a','c','a','b'])
- **THEN** Returns {'a': 3, 'b': 2, 'c': 1}

### Scenario: deque appendleft and popleft

- **GIVEN** d = deque()
- **WHEN** d.appendleft(1); d.appendleft(2); d.popleft()
- **THEN** Returns 2

</spec>
