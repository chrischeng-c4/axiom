---
id: pulsar-jieba-interfaces
type: spec
title: "Pulsar Jieba Interfaces"
version: 1
spec_type: data-model
created_at: 2026-01-30T04:26:37.418856+00:00
updated_at: 2026-01-30T04:26:37.418856+00:00
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
      title: "Pulsar Jieba Class Diagram"
history:
  - timestamp: 2026-01-30T04:26:37.418856+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Pulsar Jieba Interfaces

## Overview

This specification defines the public interfaces and data structures for the `cclab-pulsar-jieba` library. It covers the core token structure and the functional interfaces for tokenization, keyword extraction, and POS tagging.

## Requirements

### R1 - Token Data Structure

```yaml
id: R1
priority: high
status: draft
```

A Token structure containing word, start/end offsets, and optional POS tag.

### R2 - Tokenization Interface

```yaml
id: R2
priority: high
status: draft
```

Function to tokenize text into a vector of Tokens based on the selected mode.

### R3 - Keyword Extraction Interface

```yaml
id: R3
priority: medium
status: draft
```

Function to extract keywords with associated TF-IDF scores.

### R4 - POS Tagging Interface

```yaml
id: R4
priority: medium
status: draft
```

Function to tag text and return Tokens with POS information.

## Acceptance Criteria

### Scenario: Tokenize with Offsets

- **WHEN** The text "我来到" is tokenized.
- **THEN** The first token should be "我" with start 0 and end 3.

### Scenario: Keyword Extraction with Weights

- **WHEN** Keyword extraction is run.
- **THEN** The result should be a vector of Keyword objects with weights.

## Diagrams

### Pulsar Jieba Class Diagram

```mermaid
classDiagram
    class Token {
        +String word
        +usize start
        +usize end
        +Option<String> pos
    }
    class JiebaSegmenter {
        +tokenize(&str text, TokenizeMode mode) Vec<Token>
        +tag(&str text) Vec<Token>
    }
    class Keyword {
        +String word
        +f64 weight
    }
```

## API Specification (JSON Schema)

```yaml
$schema: http://json-schema.org/draft-07/schema#
properties:
  end:
    description: End byte offset
    type: integer
  pos:
    description: Part of speech tag
    type: string
  start:
    description: Start byte offset
    type: integer
  word:
    description: The segmented word string
    type: string
required:
- word
- start
- end
type: object
```

</spec>
