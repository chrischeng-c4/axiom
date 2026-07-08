# Task: Create Spec '{{spec_id}}'

All cclab CLI commands operate on the current working directory.

## Change ID
{{change_id}}

## Dependencies
{{dependencies}}

## Instructions

### Step 1: Analyze Code (if applicable)

If modifying existing code, analyze it first to understand the current structure:

```
score workflow analyze-code-for-spec {{change_id}} '{"file_paths":["path/to/file.py"],"analysis_type":"auto"}'
```

This returns suggested `spec_type`, `diagrams`, and `requirements` based on the code.

### Step 2: Read Context Files

Read proposal and dependency specs to maintain consistency:

```
score workflow read-artifact {{change_id}} '{"scope":"proposal"}'
score workflow read-artifact {{change_id}} '{"scope":"clarifications"}'
score workflow read-artifact {{change_id}} '{"scope":"list:specs:{{spec_id}}"}'
```

### Step 3: Determine spec_type

Based on analysis and context, choose the appropriate `spec_type`:

| spec_type | Use When | Required Elements |
|-----------|----------|-------------------|
| `http-api` | REST/HTTP endpoints | sequence diagram, OpenAPI 3.1 |
| `event-driven` | Message queues, events | sequence diagram, AsyncAPI 2.6 |
| `data-model` | Database schemas, DTOs | ERD or class diagram, JSON Schema |
| `algorithm` | Business logic, workflows | flowchart or state diagram |
| `integration` | External service integration | sequence diagram |
| `rpc-api` | MCP tools, JSON-RPC APIs | class diagram, OpenRPC 1.3 |
| `workflow` | State machines, orchestration | state or flowchart diagram, Serverless Workflow 0.8 |
| `utility` | Helper functions, utilities | (none required) |

### Step 4: Create Structured Diagrams

Use the Mermaid tool schemas for `diagrams` input. Each diagram should have:
- `type`: One of flowchart, sequence, class, state, erd, mindmap, requirement, journey
- `title`: Human-readable title
- `input`: Structured input matching the corresponding `generate_mermaid_*` tool schema

### Step 5: Include API Spec (REQUIRED for http-api, event-driven, rpc-api, workflow)

**IMPORTANT**: You MUST include the appropriate API spec for your spec_type. Natural language descriptions are NOT acceptable.

#### For `http-api` specs - OpenAPI 3.1 (REQUIRED)
#### For `event-driven` specs - AsyncAPI 2.6 (REQUIRED)
#### For `rpc-api` specs - OpenRPC 1.3 (REQUIRED)
#### For `workflow` specs - Serverless Workflow 0.8 (REQUIRED)
#### For `data-model` specs - JSON Schema in data_model field

### Step 6: Create Spec

Call `create_spec` with ALL required fields:

```
score artifact create-spec {{change_id}} <payload_path>
```

Where `<payload_path>` is a JSON file containing:
```json
{
  "spec_id": "{{spec_id}}",
  "title": "Human-readable title",
  "overview": "What this spec covers (min 50 chars)",
  "spec_type": "http-api",
  "requirements": [
    {"id": "R1", "title": "...", "description": "...", "priority": "high"},
    {"id": "R2", "title": "...", "description": "...", "priority": "medium"}
  ],
  "scenarios": [
    {"name": "...", "given": "...", "when": "...", "then": "..."},
    {"name": "...", "when": "...", "then": "..."}
  ],
  "diagrams": [
    {"type": "sequence", "title": "...", "input": {...}},
    {"type": "flowchart", "title": "...", "input": {...}}
  ],
  "api_spec": {"type": "openapi-3.1", "spec": {...}}
}
```

### Step 7: Validate Completeness

After creating the spec, validate it:

```
score workflow validate-spec-completeness {{change_id}} {{spec_id}}
```

If validation fails, fix the missing elements and update the spec.

## Expected Output
- specs/{{spec_id}}.md created via `score artifact create-spec`
- Spec passes `score workflow validate-spec-completeness` check

## Spec Type Requirements Summary

| spec_type | Required Diagrams | Required API Spec |
|-----------|-------------------|-------------------|
| http-api | sequence | OpenAPI 3.1 |
| event-driven | sequence | AsyncAPI 2.6 |
| data-model | erd OR class | JSON Schema |
| algorithm | flowchart OR state | - |
| integration | sequence | - |
| rpc-api | class | OpenRPC 1.3 |
| workflow | state OR flowchart | Serverless Workflow 0.8 |
| utility | - | - |
