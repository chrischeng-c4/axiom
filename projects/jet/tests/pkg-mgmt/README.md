# Block: package management — replace npm/pnpm, and be faster

**Claim.** `jet install/add/remove/update/audit` fully replaces npm/pnpm for
real projects: fixture hydration, lockfile (`jet-lock.yaml`), workspace
linking, executable bins, mutation behavior, registry/integrity handling —
and beats npm/pnpm on cold install, warm install, and disk footprint.

**Replacement rule.** npm/pnpm may run only as isolated benchmark comparators
(provisioned copies), never as Jet's fixture or runtime manager. No gate may
use `npm ci` as the fixture path.

## Gates

**The block owner is `pkg_replacement_gate.rs`**: run
`cargo test -p jet --test pkg_replacement_gate -- --nocapture` to verify the
whole claim. It drives `compare-pkg-management.mjs` with required npm/pnpm
baselines and asserts (a) every replacement-contract check is green and
(b) jet's cold and warm installs are strictly faster than the fastest
incumbent on every benchmark fixture. It skips with a message when
node/npm/pnpm are unavailable.

| Gate | Command | Covers |
|---|---|---|
| Block owner | `cargo test -p jet --test pkg_replacement_gate -- --nocapture` | full replacement contract + strict cold/warm speed win vs npm/pnpm |
| Unit/contract suite | `cargo test -p jet --lib pkg_manager -- --nocapture` | resolver, lockfile, mutation, bins, integrity |
| Workspace protocol | `cargo test -p jet --test workspace_protocol` | pnpm-style `workspace:` protocol linking |
| Replacement + speed benchmark | `JET_BASIC_DOM_PACKAGE_BASELINES=npm,pnpm JET_BASIC_DOM_REQUIRE_BASELINES=1 projects/jet/scripts/verify-basic-dom-gates.sh --phase package` | same contract via the phase script (`compare-pkg-management.mjs`) |

Benchmark fixtures live under `../fixtures/dom-production-build/`
(`react-bench`, `mui-visual`, `antd-visual`, ...).

## In this folder

- `pkg_replacement_gate.rs` — block owner; replacement + strict speed gate
  vs npm/pnpm (see above).
- `workspace_protocol.rs` — workspace protocol (pnpm-style) integration tests.

Most package-manager coverage is `--lib pkg_manager` unit/contract tests plus
the script gate above; new integration-level package tests belong here.

## Open gaps before "full replacement" is claimable

- Broader lifecycle-script output, cache evidence, and workspace layout
  breadth in the Jet-owned gate.
- Registry/auth edge cases (scoped registries, tokens) are not yet gated.
