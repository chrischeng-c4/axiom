# Task: Review Spec '{{spec_id}}' (Iteration {{iteration}})

All cclab CLI commands operate on the current working directory.

## Change ID
{{change_id}}

## Instructions

### Step 1: Validate Spec Completeness

First, run the automated validation:

```
score workflow validate-spec-completeness {{change_id}} {{spec_id}}
```

Check the output for:
- `is_complete`: Whether all required elements are present
- `missing_elements`: List of missing required elements
- `warnings`: Recommendations for improvement
- `coverage`: Requirements-to-scenarios ratio

### Step 2: Read Context

Get all context for manual review:

```
score workflow read-artifact {{change_id}} '{"scope":"{{spec_id}}"}'
score workflow read-artifact {{change_id}} '{"scope":"proposal"}'
score workflow read-artifact {{change_id}} '{"scope":"list:specs:{{spec_id}}"}'
```

### Step 3: Review Logic

#### Check 1: Is spec_type correct?

| If change involves... | Correct spec_type |
|-----------------------|-------------------|
| REST endpoints, HTTP APIs | `http-api` |
| Message queues, pub/sub, events | `event-driven` |
| Database schemas, DTOs, data structures | `data-model` |
| Business logic, computations, algorithms | `algorithm` |
| External service calls, third-party APIs | `integration` |
| State machines, orchestration, startup/shutdown | `workflow` |
| MCP tools, JSON-RPC | `rpc-api` |
| Pure helper functions (no state/flow/API) | `utility` |

#### Check 2: Are required elements present?

| spec_type | Required Diagram | Required API Spec |
|-----------|------------------|-------------------|
| `http-api` | sequence | OpenAPI 3.1 |
| `event-driven` | sequence | AsyncAPI 2.6 |
| `data-model` | erd OR class | JSON Schema |
| `algorithm` | flowchart OR state | - |
| `integration` | sequence | - |
| `workflow` | state OR flowchart | Serverless Workflow 0.8 |
| `rpc-api` | class | OpenRPC 1.3 |
| `utility` | - | - |

#### Check 3: Content quality

- **Requirements**: Are they testable and specific?
- **Scenarios**: Do they cover happy path, errors, edge cases?
- **Diagrams**: Is syntax correct and logic accurate?
- **API Spec**: Is schema complete and matches requirements?

### Step 4: Submit Review

```
score artifact review-spec {{change_id}} <payload_path>
```

## Verdict Guidelines

- **APPROVED**: Spec passes validation AND manual review
- **REVIEWED**: Missing elements, unclear requirements, insufficient scenarios
- **REJECTED**: Fundamental design problems, wrong spec_type
