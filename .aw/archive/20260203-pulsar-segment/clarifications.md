---
change: pulsar-jieba
date: 2026-01-30
---

# Clarifications

## Q1: Seg Modes
- **Question**: Which segmentation modes should we support?
- **Answer**: All 3 modes - Precise, Full, and Search
- **Rationale**: Matches original jieba API for full compatibility. Precise for accuracy, Full for all possible words, Search for search engine indexing.

## Q2: Dict Data
- **Question**: How should we handle the dictionary data?
- **Answer**: Embedded - compile dict into binary
- **Rationale**: Zero runtime I/O, self-contained binary. Larger binary size is acceptable tradeoff for simplicity and reliability.

## Q3: Keywords
- **Question**: Which keyword extraction methods should we include?
- **Answer**: TF-IDF only
- **Rationale**: Simpler and faster, covers most use cases. TextRank can be added in future iteration if needed.

## Q4: POS Tagging
- **Question**: Should we include part-of-speech (POS) tagging?
- **Answer**: Yes, include POS tagging
- **Rationale**: Common jieba feature, important for NLP applications. Uses HMM-based tagging.

