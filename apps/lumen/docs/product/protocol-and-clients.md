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

## Generated-client protocol parity

- Problem: Generated source needs typed streaming and errors.
- Who: TypeScript, Python, and Rust consumers.
- Promise: Generated clients represent the supported protocol consistently.
- Outcome: `generated-client-protocol-parity`. Tracking: Not assigned.
- Non-goals: Published packages.
- Open: Define typed streaming and errors for each language.
- Neighbours: Strict generated-client gates.

## Strict generated-client gates

- Problem: Missing local toolchains can hide a skipped client journey.
- Who: Release engineers and client consumers.
- Promise: Required language gates fail when a required client does not run.
- Outcome: `strict-generated-client-gates`. Tracking: Not assigned.
- Non-goals: Optional silent skips.
- Open: Define the required toolchain matrix.
- Neighbours: Generated-client protocol parity.

## Non-goals in this area

Lumen does not publish generated SDK packages today.
