# Protocol and clients

## Protocol and clients

- Problem: Callers need a declared wire contract and generated source boundaries.
- Who: API and generated-client consumers.
- Promise: Lumen declares current HTTP shapes and can generate supported client source.
- Status rows: `openapi-declared-contract`, `protocol-contract-completeness`, `generated-client-source-generation`, `generated-client-runtime-parity`, `generated-client-streaming-and-errors`, `generated-client-request-resilience`, `generated-client-source-integration-helpers`, `versioned-client-workload-template`, `protocol-compatibility-policy`, `published-generated-sdk-packages`, `generated-client-search-v2-parity`.
- Limits today: The current generated clients do not promise complete streaming, resilience, Search v2 parity, or published packages.
- Non-goals: Published SDK packages.
- Neighbours: Querying owns Search v2 semantics; access owns credential behavior.

## Protocol contract completeness (Milestone #12)

- Problem: Declared shapes omit some cross-request behavior.
- Who: HTTP callers.
- Promise: OpenAPI and maintained guides describe the complete supported protocol.
- Outcome: `protocol-contract-completeness`. Tracking: [Milestone #12](https://github.com/chrischeng-c4/axiom/milestone/12).
- Non-goals: Undocumented compatibility guesses.
- Open: Complete shared errors, streaming, and consistency declarations.
- Neighbours: Protocol compatibility policy.

## Generated-client protocol parity (Milestone #13)

- Problem: Generated source needs typed streaming and errors.
- Who: TypeScript, Python, and Rust consumers.
- Promise: Generated clients represent the supported protocol consistently.
- Outcome: `generated-client-protocol-parity`. Tracking: [Milestone #13](https://github.com/chrischeng-c4/axiom/milestone/13).
- Non-goals: Published packages.
- Open: Define typed streaming and errors for each language.
- Neighbours: Strict generated-client gates.

## Strict generated-client gates (Milestone #13)

- Problem: Missing local toolchains can hide a skipped client journey.
- Who: Release engineers and client consumers.
- Promise: Required language gates fail when a required client does not run.
- Outcome: `strict-generated-client-gates`. Tracking: [Milestone #13](https://github.com/chrischeng-c4/axiom/milestone/13).
- Non-goals: Optional silent skips.
- Open: Define the required toolchain matrix.
- Neighbours: Generated-client protocol parity.

## Generated-client Search v2 parity

- Problem: Search v2 callers need typed request and response parity in every generated client.
- Who: TypeScript, Python, and Rust client users.
- Promise: Generated clients expose the complete Search v2 contract as typed APIs.
- Outcome: `generated-client-search-v2-parity`. Tracking: Not assigned.
- Non-goals: Untyped JSON fallback for new unions.
- Open: Define all language-specific Search v2 parity gates.
- Neighbours: Unified search contract and strict generated-client gates.

## Generated-client request resilience (Milestone #14)

- Problem: Callers need safe retry, deadline, and cancellation behavior.
- Who: Generated-client consumers.
- Promise: Clients apply an operation-aware request-resilience contract.
- Outcome: `generated-client-request-resilience`. Tracking: [Milestone #14](https://github.com/chrischeng-c4/axiom/milestone/14).
- Non-goals: Retrying ambiguous writes without a contract.
- Open: Define retry and timeout policy.
- Neighbours: Idempotent write replay.

## Generated-client source-integration helpers (Milestone #14)

- Problem: Every caller repeats ID hydration and result ordering.
- Who: Application integrations.
- Promise: Generated clients can help bulk-fetch source records and restore hit order.
- Outcome: `generated-client-source-integration-helpers`. Tracking: [Milestone #14](https://github.com/chrischeng-c4/axiom/milestone/14).
- Non-goals: Storing source records in Lumen.
- Open: Define the callback contract.
- Neighbours: Current indexing and querying boundaries.

## Versioned client workload template

- Problem: Managed client workloads need a clear template boundary.
- Who: Kubernetes application teams.
- Promise: Lumen provides a versioned client-workload template.
- Outcome: `versioned-client-workload-template`. Tracking: Not assigned.
- Non-goals: Fleet creating client deployments.
- Open: Define projection and upgrade inputs.
- Neighbours: Managed KSA access.

## Protocol compatibility policy

- Problem: HTTP changes need a published compatibility rule.
- Who: API and client maintainers.
- Promise: Lumen defines additive, deprecated, and breaking protocol changes.
- Outcome: `protocol-compatibility-policy`. Tracking: Not assigned.
- Non-goals: Published packages.
- Open: Define release-note and overlap requirements.
- Neighbours: Search v2 migration.

## Search v2 migration

- Problem: Callers need an explicit path from the current search contract to Search v2.
- Who: Existing Lumen callers.
- Promise: Callers can use documented compatibility steps and offline tools before activation.
- Outcome: `search-v2-migration`. Tracking: Not assigned.
- Non-goals: Activating Search v2 before all members support it.
- Open: Define the full migration-tool and compatibility contract.
- Neighbours: Search capability activation at `lumen@0.37.0`.

## Non-goals in this area

Lumen does not publish generated SDK packages today.
