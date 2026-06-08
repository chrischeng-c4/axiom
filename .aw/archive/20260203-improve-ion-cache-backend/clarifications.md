---
change: improve-ion-cache-backend
date: 2026-01-30
---

# Clarifications

## Q1: Priority Order
- **Question**: What is your priority order for implementing these features?
- **Answer**: All in parallel - implement all features simultaneously
- **Rationale**: User wants comprehensive cache/result backend support in one release

## Q2: Eviction Policy
- **Question**: What memory eviction policy should be the default?
- **Answer**: allkeys-lru - Evict least recently used keys when memory full
- **Rationale**: LRU is industry standard for cache systems, balances simplicity and effectiveness

## Q3: Blocking Operations
- **Question**: Should BRPOP (blocking pop) support timeout?
- **Answer**: Yes with timeout - BRPOP key timeout returns after timeout if no data
- **Rationale**: Timeout prevents indefinite blocking, matches Redis behavior for task queue use cases

## Q4: Hash Depth
- **Question**: Should Hash operations support nested values?
- **Answer**: Nested JSON - Allow KvValue (including Map/List) as field values
- **Rationale**: Minimal performance impact since KvValue already supports Map/List, provides flexibility for structured task metadata

