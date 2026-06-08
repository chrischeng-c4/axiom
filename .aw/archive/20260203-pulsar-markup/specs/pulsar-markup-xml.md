---
id: pulsar-markup-xml
type: spec
title: "XML Parser"
version: 1
spec_type: utility
created_at: 2026-01-30T07:20:00.000000+00:00
updated_at: 2026-01-30T07:20:00.000000+00:00
requirements:
  total: 3
  ids: [R1, R2, R3]
---

<spec>

# XML Parser

## Overview

Strict XML parser with namespace support.

## Requirements

### R1 - XML Tokenizer

```yaml
id: R1
priority: high
status: draft
```

Implement XML tokenizer:
- Elements, attributes, text, CDATA
- Processing instructions
- Entity references (&amp;, &lt;, etc.)
- Namespace prefixes

### R2 - Namespace Support

```yaml
id: R2
priority: high
status: draft
```

Handle XML namespaces:
- Default namespace (`xmlns="..."`)
- Prefixed namespaces (`xmlns:prefix="..."`)
- Namespace-aware element/attribute lookup

### R3 - XML Serialization

```yaml
id: R3
priority: medium
status: draft
```

Serialize DOM back to XML:
- Pretty-print with indentation
- Compact mode (no whitespace)
- Preserve namespaces

## Acceptance Criteria

### Scenario: Parse XML with Namespaces

- **GIVEN** XML with multiple namespaces
- **WHEN** Call `parse_xml()`
- **THEN** Namespaces are resolved correctly

</spec>
