# Task: Revise Spec '{{spec_id}}' Based on Review Feedback

All cclab CLI commands operate on the current working directory.

## Change ID
{{change_id}}

## Instructions

1. **Read the review feedback**:
   - Look for the latest review block with issues about spec '{{spec_id}}'

2. **Read current spec and dependencies**:
   - Read current spec and related specs for consistency

3. **Address each issue** using `score artifact create-spec`:
   - Fix all issues mentioned in the review for this spec
   - Ensure requirements are testable and clear
   - Ensure scenarios cover all cases
   - Maintain consistency with proposal and dependent specs
   - **IMPORTANT**: Include required formal specifications based on spec_type:
     - `http-api`: OpenAPI 3.1 in api_spec field
     - `event-driven`: AsyncAPI 2.6 in api_spec field
     - `rpc-api`: OpenRPC 1.3 in api_spec field
     - `workflow`: Serverless Workflow 0.8 in api_spec field
     - `data-model`: JSON Schema in data_model field

4. **Verify the fix**:
   - Re-read the spec to confirm issues are resolved

## Expected Output
- Updated specs/{{spec_id}}.md via `score artifact create-spec` addressing review feedback

## CLI Commands

### Read Context
```
score workflow read-artifact {{change_id}} '{"scope":"proposal"}'
score workflow read-artifact {{change_id}} '{"scope":"{{spec_id}}"}'
score workflow read-artifact {{change_id}} '{"scope":"list:specs:{{spec_id}}"}'
```

### Generate Artifact
```
score artifact create-spec {{change_id}} <payload_path>
```

## Spec Type Requirements

| spec_type | Required Diagrams | Required API Spec |
|-----------|-------------------|-------------------|
| http-api | sequence | OpenAPI 3.1 |
| event-driven | sequence | AsyncAPI 2.6 |
| data-model | erd OR class | JSON Schema (data_model field) |
| algorithm | flowchart OR state | - |
| integration | sequence | - |
| rpc-api | class | OpenRPC 1.3 |
| workflow | state OR flowchart | Serverless Workflow 0.8 |
| utility | - | - |
