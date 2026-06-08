---
change: pulsar-bm25
date: 2026-01-30
---

# Clarifications

## Q1: Tokenizer
- **Question**: Should BM25 have a hard dependency on cclab-pulsar-jieba or use a generic tokenizer trait?
- **Answer**: Generic trait - define Tokenizer trait, jieba is one implementation
- **Rationale**: More flexible design allows BM25 to work with any tokenizer (English, Chinese, custom). Reduces coupling between crates.

## Q2: Variants
- **Question**: Which BM25 variants should we support?
- **Answer**: BM25Okapi only - standard BM25
- **Rationale**: Okapi covers 95% of use cases. Keep MVP focused, add BM25L/Plus in future iteration if needed.

## Q3: Updates
- **Question**: Should we support incremental document updates?
- **Answer**: Batch only - index all docs at once
- **Rationale**: Simpler implementation, faster for typical search use cases. Incremental updates add complexity for marginal benefit.

