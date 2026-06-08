# Query API

## Overview
<!-- type: overview lang: markdown -->

Fluent query builder API for CtxInfDB. Supports entity/relation CRUD, graph traversal queries, temporal queries, and inference triggers. Returns typed result sets with cursor-based pagination for large results.

## Query Builder Interface
<!-- type: rpc-api lang: json -->

```json
{
  "openrpc": "1.3.0",
  "info": {
    "title": "CtxInfDB Query API",
    "version": "0.1.0"
  },
  "methods": [
    {
      "name": "entity.create",
      "summary": "Create a new entity",
      "params": [
        { "name": "entity_type", "schema": { "$ref": "data-model#entity-type" }, "required": true },
        { "name": "name", "schema": { "type": "string" }, "required": true },
        { "name": "properties", "schema": { "type": "object" } },
        { "name": "valid_from", "schema": { "type": "string", "format": "date-time" } },
        { "name": "valid_to", "schema": { "type": "string", "format": "date-time" } },
        { "name": "source_refs", "schema": { "type": "array", "items": { "type": "string" } } }
      ],
      "result": { "name": "entity", "schema": { "$ref": "data-model#entity" } }
    },
    {
      "name": "entity.get",
      "summary": "Get entity by ID",
      "params": [
        { "name": "id", "schema": { "type": "string", "format": "uuid" }, "required": true }
      ],
      "result": { "name": "entity", "schema": { "$ref": "data-model#entity" } }
    },
    {
      "name": "entity.find",
      "summary": "Find entities by filter",
      "params": [
        { "name": "entity_type", "schema": { "$ref": "data-model#entity-type" } },
        { "name": "name_contains", "schema": { "type": "string" } },
        { "name": "properties", "schema": { "type": "object" } },
        { "name": "active_at", "schema": { "type": "string", "format": "date-time" } },
        { "name": "active_during", "schema": { "type": "object", "properties": { "from": { "type": "string", "format": "date-time" }, "to": { "type": "string", "format": "date-time" } } } },
        { "name": "limit", "schema": { "type": "integer", "default": 100 } },
        { "name": "cursor", "schema": { "type": "string" } }
      ],
      "result": { "name": "page", "schema": { "$ref": "#/components/schemas/EntityPage" } }
    },
    {
      "name": "entity.update",
      "summary": "Update an existing entity",
      "params": [
        { "name": "id", "schema": { "type": "string", "format": "uuid" }, "required": true },
        { "name": "name", "schema": { "type": "string" } },
        { "name": "properties", "schema": { "type": "object" } },
        { "name": "valid_from", "schema": { "type": "string", "format": "date-time" } },
        { "name": "valid_to", "schema": { "type": "string", "format": "date-time" } },
        { "name": "source_refs", "schema": { "type": "array", "items": { "type": "string" } } },
        { "name": "version", "schema": { "type": "integer" }, "required": true, "description": "CAS version — update fails if stale" }
      ],
      "result": { "name": "entity", "schema": { "$ref": "data-model#entity" } },
      "errors": [
        { "code": -32001, "message": "EntityNotFound" },
        { "code": -32002, "message": "VersionConflict" }
      ]
    },
    {
      "name": "entity.delete",
      "summary": "Delete an entity and all its relations",
      "params": [
        { "name": "id", "schema": { "type": "string", "format": "uuid" }, "required": true },
        { "name": "cascade", "schema": { "type": "boolean", "default": true }, "description": "If true, also delete all relations involving this entity" }
      ],
      "result": { "name": "deleted", "schema": { "type": "object", "properties": { "entity_deleted": { "type": "boolean" }, "relations_deleted": { "type": "integer" } } } },
      "errors": [
        { "code": -32001, "message": "EntityNotFound" }
      ]
    },
    {
      "name": "relation.create",
      "summary": "Create a relation between two entities",
      "params": [
        { "name": "relation_type", "schema": { "$ref": "data-model#relation-type" }, "required": true },
        { "name": "source", "schema": { "type": "string", "format": "uuid" }, "required": true },
        { "name": "target", "schema": { "type": "string", "format": "uuid" }, "required": true },
        { "name": "confidence", "schema": { "type": "number", "default": 1.0 } },
        { "name": "properties", "schema": { "type": "object" } },
        { "name": "valid_from", "schema": { "type": "string", "format": "date-time" } },
        { "name": "valid_to", "schema": { "type": "string", "format": "date-time" } },
        { "name": "source_refs", "schema": { "type": "array", "items": { "type": "string" } } }
      ],
      "result": { "name": "relation", "schema": { "$ref": "data-model#relation" } }
    },
    {
      "name": "relation.find",
      "summary": "Find relations by filter",
      "params": [
        { "name": "source", "schema": { "type": "string", "format": "uuid" } },
        { "name": "target", "schema": { "type": "string", "format": "uuid" } },
        { "name": "relation_type", "schema": { "$ref": "data-model#relation-type" } },
        { "name": "min_confidence", "schema": { "type": "number" } },
        { "name": "active_at", "schema": { "type": "string", "format": "date-time" } },
        { "name": "limit", "schema": { "type": "integer", "default": 100 } },
        { "name": "cursor", "schema": { "type": "string" } }
      ],
      "result": { "name": "page", "schema": { "$ref": "#/components/schemas/RelationPage" } }
    },
    {
      "name": "relation.get",
      "summary": "Get relation by ID",
      "params": [
        { "name": "id", "schema": { "type": "string", "format": "uuid" }, "required": true }
      ],
      "result": { "name": "relation", "schema": { "$ref": "data-model#relation" } },
      "errors": [
        { "code": -32003, "message": "RelationNotFound" }
      ]
    },
    {
      "name": "relation.update",
      "summary": "Update an existing relation",
      "params": [
        { "name": "id", "schema": { "type": "string", "format": "uuid" }, "required": true },
        { "name": "confidence", "schema": { "type": "number" } },
        { "name": "properties", "schema": { "type": "object" } },
        { "name": "valid_from", "schema": { "type": "string", "format": "date-time" } },
        { "name": "valid_to", "schema": { "type": "string", "format": "date-time" } },
        { "name": "source_refs", "schema": { "type": "array", "items": { "type": "string" } } },
        { "name": "version", "schema": { "type": "integer" }, "required": true }
      ],
      "result": { "name": "relation", "schema": { "$ref": "data-model#relation" } },
      "errors": [
        { "code": -32003, "message": "RelationNotFound" },
        { "code": -32002, "message": "VersionConflict" }
      ]
    },
    {
      "name": "relation.delete",
      "summary": "Delete a relation",
      "params": [
        { "name": "id", "schema": { "type": "string", "format": "uuid" }, "required": true }
      ],
      "result": { "name": "deleted", "schema": { "type": "boolean" } },
      "errors": [
        { "code": -32003, "message": "RelationNotFound" }
      ]
    },
    {
      "name": "graph.neighbors",
      "summary": "Get neighbors of an entity",
      "params": [
        { "name": "id", "schema": { "type": "string", "format": "uuid" }, "required": true },
        { "name": "direction", "schema": { "type": "string", "enum": ["outgoing", "incoming", "both"] } },
        { "name": "relation_type", "schema": { "$ref": "data-model#relation-type" } },
        { "name": "max_hops", "schema": { "type": "integer", "default": 1 } },
        { "name": "min_confidence", "schema": { "type": "number" } },
        { "name": "active_at", "schema": { "type": "string", "format": "date-time" } }
      ],
      "result": { "name": "neighbors", "schema": { "type": "array", "items": { "$ref": "data-model#entity" } } }
    },
    {
      "name": "graph.shortest_path",
      "summary": "Find shortest path between two entities",
      "params": [
        { "name": "source", "schema": { "type": "string", "format": "uuid" }, "required": true },
        { "name": "target", "schema": { "type": "string", "format": "uuid" }, "required": true },
        { "name": "max_hops", "schema": { "type": "integer", "default": 6 } },
        { "name": "min_confidence", "schema": { "type": "number" } },
        { "name": "relation_types", "schema": { "type": "array", "items": { "$ref": "data-model#relation-type" } } },
        { "name": "active_at", "schema": { "type": "string", "format": "date-time" } }
      ],
      "result": { "name": "path", "schema": { "$ref": "graph-engine#path-result" } }
    },
    {
      "name": "graph.pagerank",
      "summary": "Compute PageRank for all or subset of entities",
      "params": [
        { "name": "entity_type", "schema": { "$ref": "data-model#entity-type" } },
        { "name": "damping", "schema": { "type": "number", "default": 0.85 } },
        { "name": "iterations", "schema": { "type": "integer", "default": 100 } },
        { "name": "active_at", "schema": { "type": "string", "format": "date-time" } }
      ],
      "result": { "name": "ranks", "schema": { "type": "array", "items": { "type": "object", "properties": { "entity_id": { "type": "string" }, "rank": { "type": "number" } } } } }
    },
    {
      "name": "graph.communities",
      "summary": "Detect communities in the graph",
      "params": [
        { "name": "algorithm", "schema": { "type": "string", "enum": ["louvain", "label_propagation"] } },
        { "name": "resolution", "schema": { "type": "number", "default": 1.0 } },
        { "name": "active_at", "schema": { "type": "string", "format": "date-time" } }
      ],
      "result": { "name": "communities", "schema": { "type": "array", "items": { "$ref": "graph-engine#community-result" } } }
    },
    {
      "name": "temporal.timeline",
      "summary": "Build timeline involving specified entities",
      "params": [
        { "name": "entity_ids", "schema": { "type": "array", "items": { "type": "string", "format": "uuid" } }, "required": true },
        { "name": "from", "schema": { "type": "string", "format": "date-time" } },
        { "name": "to", "schema": { "type": "string", "format": "date-time" } },
        { "name": "include_relations", "schema": { "type": "boolean", "default": true } }
      ],
      "result": { "name": "timeline", "schema": { "type": "array", "items": { "$ref": "temporal-engine#timeline-entry" } } }
    },
    {
      "name": "temporal.correlation",
      "summary": "Compute temporal correlation between two entities",
      "params": [
        { "name": "entity_a", "schema": { "type": "string", "format": "uuid" }, "required": true },
        { "name": "entity_b", "schema": { "type": "string", "format": "uuid" }, "required": true },
        { "name": "window_days", "schema": { "type": "integer", "default": 30 } }
      ],
      "result": { "name": "correlation", "schema": { "$ref": "temporal-engine#temporal-correlation" } }
    },
    {
      "name": "inference.scan",
      "summary": "Run inference rules across the graph",
      "params": [
        { "name": "rule_ids", "schema": { "type": "array", "items": { "type": "string" }, "description": "Specific rules to run (empty = all)" } },
        { "name": "scope_entity_ids", "schema": { "type": "array", "items": { "type": "string", "format": "uuid" }, "description": "Limit scope (empty = full graph)" } },
        { "name": "min_confidence", "schema": { "type": "number", "default": 0.3 } }
      ],
      "result": { "name": "findings", "schema": { "type": "object", "properties": { "inferred_relations": { "type": "integer" }, "anomalies": { "type": "array", "items": { "$ref": "inference-engine#anomaly-report" } } } } }
    },
    {
      "name": "inference.explain",
      "summary": "Explain why a relation was inferred",
      "params": [
        { "name": "relation_id", "schema": { "type": "string", "format": "uuid" }, "required": true }
      ],
      "result": { "name": "explanation", "schema": { "type": "object", "properties": { "rule_id": { "type": "string" }, "pattern_match": { "type": "object" }, "confidence_breakdown": { "type": "object" }, "evidence_chain": { "type": "array" } } } }
    }
  ],
  "components": {
    "schemas": {
      "EntityPage": {
        "type": "object",
        "properties": {
          "items": { "type": "array", "items": { "$ref": "data-model#entity" } },
          "next_cursor": { "type": "string" },
          "total_estimate": { "type": "integer" }
        }
      },
      "RelationPage": {
        "type": "object",
        "properties": {
          "items": { "type": "array", "items": { "$ref": "data-model#relation" } },
          "next_cursor": { "type": "string" },
          "total_estimate": { "type": "integer" }
        }
      }
    }
  }
}
```
