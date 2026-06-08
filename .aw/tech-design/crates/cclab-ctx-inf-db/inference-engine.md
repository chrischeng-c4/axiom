# Inference Engine

## Overview
<!-- type: overview lang: markdown -->

Rule-based inference engine for detecting patterns, propagating confidence, and scoring anomalies in the temporal knowledge graph. Designed for intelligence analysis: identifying indirect connections, suspicious temporal clusters, and hidden influence networks.

## Inference Pipeline
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TD
    trigger[Trigger: new entity/relation inserted OR manual scan] --> load_rules[Load active rule set]
    load_rules --> match[Pattern Matching Phase]
    match --> candidates[Candidate matches]
    candidates --> score[Confidence Scoring Phase]
    score --> threshold{Confidence >= threshold?}
    threshold -->|yes| materialize[Materialize inferred relations]
    threshold -->|no| discard[Discard low-confidence]
    materialize --> propagate[Propagation Phase]
    propagate --> check_cascade{New inferences triggered?}
    check_cascade -->|yes| match
    check_cascade -->|no| anomaly[Anomaly Detection Phase]
    anomaly --> report[Generate findings report]
```

## Rule Engine Architecture
<!-- type: dependency lang: mermaid -->

```mermaid
classDiagram
    direction TB

    class RuleEngine {
        rules: Vec~Rule~
        +register(rule)
        +evaluate(trigger) Vec~InferredRelation~
        +scan_all() Vec~InferredRelation~
    }

    class Rule {
        <<trait>>
        +id() String
        +description() String
        +pattern() Pattern
        +confidence_fn() ConfidenceFn
        +evaluate(graph, trigger) Vec~Match~
    }

    class Pattern {
        nodes: Vec~NodeConstraint~
        edges: Vec~EdgeConstraint~
        temporal: Option~TemporalConstraint~
    }

    class NodeConstraint {
        variable: String
        entity_type: Option~EntityType~
        properties: Map~String_ValuePredicate~
    }

    class EdgeConstraint {
        source_var: String
        target_var: String
        relation_type: Option~RelationType~
        min_confidence: Option~f64~
        direction: Direction
    }

    class TemporalConstraint {
        window: Duration
        ordering: Option~Ordering~
        overlap_required: bool
    }

    class ConfidenceScorer {
        +path_decay(hops, base) f64
        +evidence_weight(source_count) f64
        +temporal_proximity(gap, window) f64
        +combined(factors) f64
    }

    class AnomalyDetector {
        +structural_anomaly(entity) f64
        +temporal_anomaly(entity) f64
        +behavioral_anomaly(entity) f64
        +composite_score(entity) AnomalyReport
    }

    RuleEngine --> Rule
    Rule --> Pattern
    Pattern --> NodeConstraint
    Pattern --> EdgeConstraint
    Pattern --> TemporalConstraint
    RuleEngine --> ConfidenceScorer
    RuleEngine --> AnomalyDetector
```

## Confidence Propagation
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TD
    subgraph HopDecay["Hop Decay"]
        direct[Direct evidence: confidence = 1.0] --> hop1[1-hop inference]
        hop1 --> calc1["confidence = base * decay^1"]
        calc1 --> hop2[2-hop inference]
        hop2 --> calc2["confidence = base * decay^2"]
        calc2 --> hop3[3-hop inference]
        hop3 --> calc3["confidence = base * decay^3"]
        calc3 --> cutoff{confidence < min_threshold?}
        cutoff -->|yes| stop[Stop propagation]
        cutoff -->|no| continue[Continue to next hop]
    end

    subgraph EvidenceBoost["Evidence Combination"]
        evidence[Multiple evidence sources] --> combine["combined = 1 - Π(1 - ci)"]
        combine --> boost[Evidence-boosted confidence]
    end

    subgraph TemporalDecay["Temporal Proximity"]
        temporal_gap[Temporal proximity] --> decay_t["temporal_factor = e^(-gap/window)"]
        decay_t --> adjust[Apply temporal adjustment]
    end

    HopDecay --> final[Final confidence]
    EvidenceBoost --> final
    TemporalDecay --> final
    final --> output["confidence = hop_score * evidence_boost * temporal_factor"]
```

## Rule Definition Schema
<!-- type: schema lang: json -->

```json
{
  "$id": "inference-rule",
  "title": "InferenceRule",
  "type": "object",
  "required": ["id", "description", "pattern", "infers"],
  "properties": {
    "id": { "type": "string", "pattern": "^R-[A-Z]+-\\d+$" },
    "description": { "type": "string" },
    "enabled": { "type": "boolean", "default": true },
    "priority": { "type": "integer", "minimum": 0, "maximum": 100 },
    "pattern": {
      "type": "object",
      "properties": {
        "nodes": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "var": { "type": "string" },
              "entity_type": { "$ref": "data-model#entity-type" },
              "properties": { "type": "object" }
            }
          }
        },
        "edges": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "from": { "type": "string" },
              "to": { "type": "string" },
              "relation_type": { "$ref": "data-model#relation-type" },
              "min_confidence": { "type": "number" }
            }
          }
        },
        "temporal": {
          "type": "object",
          "properties": {
            "window_days": { "type": "integer" },
            "ordering": { "type": "string", "enum": ["before", "after", "concurrent", "any"] },
            "overlap_required": { "type": "boolean" }
          }
        }
      }
    },
    "infers": {
      "type": "object",
      "description": "The relation to create when pattern matches",
      "properties": {
        "relation_type": { "$ref": "data-model#relation-type" },
        "source_var": { "type": "string" },
        "target_var": { "type": "string" },
        "base_confidence": { "type": "number", "minimum": 0, "maximum": 1 },
        "decay_per_hop": { "type": "number", "default": 0.15 }
      }
    }
  }
}
```

## Example Rules
<!-- type: overview lang: markdown -->

**R-AFF-001: Transitive Affiliation**
If A is `member_of` Org1, and Org1 is `affiliated_with` Org2, infer A is `affiliated_with` Org2 (confidence decayed by hop).

**R-MTG-001: Meeting Network**
If A `met_with` B, and B `met_with` C within 30 days, and A never directly met C, flag as potential indirect coordination.

**R-FND-001: Follow the Money**
If Org1 `funded` Org2, and Person is `member_of` Org2, infer Person is `influenced_by` Org1 (base confidence 0.6, decays).

**R-TMP-001: Suspicious Temporal Clustering**
If Person has 3+ `met_with` relations within 7 days preceding a `policy` entity's `valid_from`, flag as anomaly.

**R-ANO-001: Bridge Node Detection**
If a Person has high betweenness centrality AND connects two otherwise disconnected communities, flag as potential intermediary.

## Anomaly Report Schema
<!-- type: schema lang: json -->

```json
{
  "$id": "anomaly-report",
  "title": "AnomalyReport",
  "type": "object",
  "properties": {
    "entity_id": { "type": "string", "format": "uuid" },
    "entity_name": { "type": "string" },
    "composite_score": {
      "type": "number",
      "minimum": 0.0,
      "maximum": 1.0,
      "description": "Weighted combination of all anomaly dimensions"
    },
    "dimensions": {
      "type": "object",
      "properties": {
        "structural": {
          "type": "number",
          "description": "Unusual graph position (high betweenness, bridge node)"
        },
        "temporal": {
          "type": "number",
          "description": "Unusual temporal patterns (clustered activity)"
        },
        "behavioral": {
          "type": "number",
          "description": "Deviation from peer group behavior"
        }
      }
    },
    "triggered_rules": {
      "type": "array",
      "items": { "type": "string", "description": "Rule IDs that flagged this entity" }
    },
    "evidence": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "rule_id": { "type": "string" },
          "description": { "type": "string" },
          "related_entities": {
            "type": "array",
            "items": { "type": "string", "format": "uuid" }
          },
          "confidence": { "type": "number" }
        }
      }
    }
  }
}
```

## Inference State Machine
<!-- type: state-machine lang: mermaid -->

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Triggered : new data / manual scan
    Triggered --> Matching : load rules
    Matching --> Scoring : candidates found
    Matching --> Idle : no matches
    Scoring --> Materializing : above threshold
    Scoring --> Idle : all below threshold
    Materializing --> Propagating : inferred relations written
    Propagating --> Matching : cascade triggered
    Propagating --> AnomalyCheck : no cascade
    AnomalyCheck --> Reporting : anomalies detected
    AnomalyCheck --> Idle : clean
    Reporting --> Idle : report generated
```
