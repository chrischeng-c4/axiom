# Temporal Engine

> **Phase 1 scope marker.** `src/temporal.rs` (Phase 1) currently provides `active_during`, `active_at`, `relations_at`, and `timeline` as direct methods on `impl CtxInfEngine`, backed by linear DashMap iteration + `TemporalRange::{contains, overlaps}`. The `TemporalOps` trait, `IntervalIndex` B-tree, `TimelineBuilder`, `EventSequencer`, `TemporalCorrelation`, and interval-index flow below are **Phase 3+ aspirational** — they describe the target indexed architecture, not today's code. The "Timeline Entry Schema" IS implemented in Phase 1 (minus the `description` field, which is reserved for Phase 3).

## Overview
<!-- type: overview lang: markdown -->

Temporal engine for time-aware graph queries. Every entity and relation carries `(valid_from, valid_to)` interval bounds. The engine provides interval indexing (B-tree on intervals), point-in-time graph snapshots, timeline construction, and temporal correlation detection across entities.

## Temporal Query Types
<!-- type: dependency lang: mermaid -->

```mermaid
classDiagram
    direction TB

    class TemporalOps {
        <<trait>>
        +at_time(t) GraphSnapshot
        +during(range) GraphSlice
        +timeline(entity_ids) Timeline
    }

    class IntervalIndex {
        +insert(id, from, to)
        +remove(id)
        +overlapping(range) Vec~id~
        +containing(point) Vec~id~
        +active_at(point) Vec~id~
    }

    class TimelineBuilder {
        +involving(entity_ids) Self
        +date_range(from, to) Self
        +include_relations(bool) Self
        +build() Timeline
    }

    class EventSequencer {
        +sequence(events) Vec~Event~
        +causal_chain(event_id) Vec~Event~
        +concurrent_events(event_id, window) Vec~Event~
    }

    class TemporalCorrelation {
        +co_occurrence(a, b, window) f64
        +precedes_frequency(a, b, window) f64
        +temporal_clustering(events) Vec~Cluster~
    }

    TemporalOps <|.. TimelineBuilder
    TemporalOps <|.. EventSequencer
    TimelineBuilder --> IntervalIndex
    EventSequencer --> IntervalIndex
    TemporalCorrelation --> IntervalIndex
    TemporalCorrelation --> EventSequencer
```

## Interval Index Structure
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TD
    subgraph BTree["B-Tree on valid_from"]
        root[Root Node]
        root --> l1[Internal Node]
        root --> l2[Internal Node]
        l1 --> leaf1[Leaf: entries sorted by valid_from]
        l1 --> leaf2[Leaf: entries sorted by valid_from]
        l2 --> leaf3[Leaf: entries sorted by valid_from]
    end

    query_point[Point Query: t=2024-03-15] --> scan[Scan leaves where valid_from <= t]
    scan --> filter[Filter: valid_to >= t OR valid_to = NULL]
    filter --> result[Active entities at t]

    query_range[Range Query: 2024-01..2024-06] --> scan_range[Scan leaves where valid_from <= range.end]
    scan_range --> filter_range[Filter: valid_to >= range.start OR valid_to = NULL]
    filter_range --> result_range[Overlapping entities]
```

## Timeline Construction
<!-- type: interaction lang: mermaid -->

```mermaid
sequenceDiagram
    participant C as Caller
    participant TB as TimelineBuilder
    participant TI as IntervalIndex
    participant BE as BufferPool
    participant E as Engine

    C->>TB: timeline().involving([A, B, C]).date_range(2020, 2025).build()
    TB->>TI: overlapping(2020..2025) for entity A
    TI-->>TB: entity A intervals
    TB->>TI: overlapping(2020..2025) for entity B
    TI-->>TB: entity B intervals
    TB->>TI: overlapping(2020..2025) for entity C
    TI-->>TB: entity C intervals
    TB->>E: load_relations(between [A,B,C], 2020..2025)
    E->>BE: fetch edge pages
    BE-->>E: edge data
    E-->>TB: relations with temporal ranges
    TB->>TB: merge-sort all events by timestamp
    TB->>TB: annotate with relation context
    TB-->>C: Timeline { entries: [...] }
```

## Timeline Entry Schema
<!-- type: schema lang: json -->

```json
{
  "$id": "timeline-entry",
  "title": "TimelineEntry",
  "type": "object",
  "required": ["timestamp", "entry_type"],
  "properties": {
    "timestamp": { "type": "string", "format": "date-time" },
    "entry_type": {
      "type": "string",
      "enum": ["entity_start", "entity_end", "relation_start", "relation_end", "event_occurred"]
    },
    "entity_id": { "type": "string", "format": "uuid" },
    "entity_name": { "type": "string" },
    "entity_type": { "$ref": "data-model#entity-type" },
    "relation_id": { "type": "string", "format": "uuid" },
    "relation_type": { "$ref": "data-model#relation-type" },
    "counterpart_id": { "type": "string", "format": "uuid" },
    "counterpart_name": { "type": "string" },
    "description": { "type": "string", "description": "Reserved for Phase 3+ enrichment; absent from Phase 1 src/temporal.rs::TimelineEntry." }
  }
}
```

## Temporal Correlation Schema
<!-- type: schema lang: json -->

```json
{
  "$id": "temporal-correlation",
  "title": "TemporalCorrelation",
  "type": "object",
  "properties": {
    "entity_a": { "type": "string", "format": "uuid" },
    "entity_b": { "type": "string", "format": "uuid" },
    "co_occurrence_score": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 1.0,
      "description": "How often A and B are active in the same time window"
    },
    "precedes_score": {
      "type": "number",
      "description": "How often A's events precede B's within a window"
    },
    "window_days": {
      "type": "integer",
      "description": "Temporal window used for correlation"
    },
    "sample_count": {
      "type": "integer",
      "description": "Number of event pairs examined"
    }
  }
}
```
