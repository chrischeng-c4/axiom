---
id: pulsar-markup-xpath
type: spec
title: "XPath Engine"
version: 1
spec_type: utility
created_at: 2026-01-30T07:20:00.000000+00:00
updated_at: 2026-01-30T07:20:00.000000+00:00
requirements:
  total: 3
  ids: [R1, R2, R3]
---

<spec>

# XPath Engine

## Overview

XPath 1.0 query engine for XML/HTML documents.

## Requirements

### R1 - Path Expressions

```yaml
id: R1
priority: high
status: draft
```

Support XPath axes:
- `/root/child` - absolute path
- `//descendant` - anywhere
- `./relative` - from context
- `..` - parent
- `@attribute` - attribute access

### R2 - Predicates

```yaml
id: R2
priority: high
status: draft
```

Support predicates:
- Position: `[1]`, `[last()]`, `[position() > 1]`
- Attribute: `[@id="foo"]`, `[@class]`
- Contains: `[contains(@class, "item")]`
- Comparison: `[price > 10]`

### R3 - Functions

```yaml
id: R3
priority: medium
status: draft
```

Support XPath functions:
- String: `text()`, `string()`, `contains()`, `starts-with()`
- Node: `name()`, `local-name()`, `count()`
- Boolean: `not()`, `true()`, `false()`

## Acceptance Criteria

### Scenario: XPath Query

- **GIVEN** XML document
- **WHEN** Call `xpath("//book[@category='fiction']/title")`
- **THEN** Matching title elements returned

</spec>
