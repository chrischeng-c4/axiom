# Protocol and clients

## Protocol and clients

- Problem: Callers need a declared wire contract and generated source boundaries.
- Who: API and generated-client consumers.
- Promise: Lumen declares current HTTP shapes and can generate supported client source.
- Status rows: `openapi-declared-contract`, `protocol-contract-completeness`, `generated-client-source-generation`, `generated-client-runtime-parity`, `generated-client-streaming-and-errors`, `generated-client-request-resilience`, `generated-client-source-integration-helpers`, `versioned-client-workload-template`, `protocol-compatibility-policy`, `published-generated-sdk-packages`, `generated-client-search-v2-parity`.
- Limits today: The current generated clients do not promise complete streaming, resilience, Search v2 parity, or published packages.
- Non-goals: Published SDK packages.
- Neighbours: Querying owns Search v2 semantics; access owns credential behavior.

## Protocol contract completeness

- Problem: Declared shapes omit some cross-request behavior.
- Who: HTTP callers.
- Promise: OpenAPI and maintained guides describe the complete supported protocol.
- Outcome: `protocol-contract-completeness`. Tracking: Not assigned.
- Non-goals: Undocumented compatibility guesses.
- Open: Complete shared errors, streaming, and consistency declarations.
- Neighbours: Protocol compatibility policy.

## Non-goals in this area

Lumen does not publish generated SDK packages today.
