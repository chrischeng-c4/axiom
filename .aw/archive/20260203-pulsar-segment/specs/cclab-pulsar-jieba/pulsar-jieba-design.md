---
id: pulsar-jieba-design
type: spec
title: "Pulsar Jieba NLP Design"
version: 1
spec_type: algorithm
target_crate: cclab-pulsar-jieba
created_at: 2026-01-30T04:34:39.789859+00:00
updated_at: 2026-01-30T04:34:39.789859+00:00
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
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Chinese Segmentation Algorithm Flow"
history:
  - timestamp: 2026-01-30T04:34:39.789859+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Pulsar Jieba NLP Design

## Overview

This specification defines the technical design for `cclab-pulsar-jieba`, a high-performance Chinese tokenization library. It implements the classic Jieba algorithm in pure Rust, providing multi-mode segmentation, HMM-based unknown word recognition, TF-IDF keyword extraction, and POS tagging. The library is designed to be zero-dependency (relative to external NLP crates) and integrates with `cclab-nucleus` for Python accessibility.

## Requirements

### R1 - Segmentation Modes

```yaml
id: R1
priority: medium
status: draft
```

Support Precise (minimal segments), Full (all possible segments), and Search (long words + granular segments) modes.

### R2 - DAG-based Path Finding

```yaml
id: R2
priority: medium
status: draft
```

Use a high-performance Trie-based Directed Acyclic Graph (DAG) for efficient dictionary lookup and path calculation.

### R3 - HMM Unknown Word Recognition

```yaml
id: R3
priority: medium
status: draft
```

Utilize the Viterbi algorithm with a Hidden Markov Model (HMM) to detect and segment out-of-vocabulary (OOV) words.

### R4 - Embedded Dictionary Support

```yaml
id: R4
priority: medium
status: draft
```

Embed a default high-quality Chinese dictionary into the binary to ensure zero-configuration usability.

### R5 - TF-IDF Keyword Extraction

```yaml
id: R5
priority: medium
status: draft
```

Implement TF-IDF algorithm for extracting top-K keywords from a given text.

### R6 - POS Tagging Support

```yaml
id: R6
priority: medium
status: draft
```

Provide HMM-based Part-of-Speech tagging for segmented words.

## Acceptance Criteria

### Scenario: Precise Segmentation of Known Sentence

- **GIVEN** The text "我来到北京清华大学" is tokenized in Precise mode.
- **WHEN** The tokenizer is called.
- **THEN** The result should be ["我", "来到", "北京", "清华大学"].

### Scenario: HMM Recognition of Unknown Words

- **GIVEN** The text "杭研大厦" is tokenized and "杭研" is not in the dictionary.
- **WHEN** The tokenizer is called.
- **THEN** The word "杭研" should be recognized as a single token even if not in the dictionary.

### Scenario: Keyword Extraction with TF-IDF

- **GIVEN** TF-IDF extraction is run on "我来到北京清华大学" with topK=2.
- **WHEN** The keyword extractor is called.
- **THEN** The result should contain ["清华大学", "北京"] among the top results.

## Diagrams

### Chinese Segmentation Algorithm Flow

```mermaid
flowchart TB
    InputText(Input Chinese Text)
    BuildDAG[Build DAG (Trie)]
    CalcPathProb[Calculate Path Probability]
    RouteSelection{Best Route Selection} 
    HMM_Viterbi[HMM Viterbi (for Unknown)]
    OutputTokens([Final Tokens])
    InputText --> BuildDAG
    BuildDAG --> CalcPathProb
    CalcPathProb --> RouteSelection
    RouteSelection -->|Known Words| OutputTokens
    RouteSelection -->|OOV Candidate| HMM_Viterbi
    HMM_Viterbi -->|Predict Word| OutputTokens
```

</spec>
