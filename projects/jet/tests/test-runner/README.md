# Block: test runner — replace Vitest/Jest with a Jet-owned TS runtime

**Claim.** `jet test` runs TS test suites with a Jet-owned `@jet/test`
runtime that is auto-available as a virtual module — no npm package install —
and covers the Vitest/Jest API surface projects actually use.

## Gates

| Gate | Command | Covers |
|---|---|---|
| Runner lib suites | `cargo test -p jet --lib test_runner -- --nocapture`, `cargo test -p jet --lib reporter -- --nocapture` | runner core and reporting |
| API compat corpus | `cargo test -p jet --test jet_test_api_compat` | `@jet/test` compat corpus under `../fixtures/jet-test-api-compat/` |
| Dogfood | `cargo test -p jet --test jet_test_dogfood` | `jet test` over the in-tree example suite |

## In this folder

- Runner end-to-end: `test_runner_smoke.rs`, `jet_test_dogfood.rs`,
  `jet_test_api_compat.rs`
- Fixture engine: `fixture_di_tests.rs`, `fixture_lifecycle_tests.rs`,
  `fixture_timeout_tests.rs`
- Execution infra: `worker_pool_tests.rs`, `web_server_tests.rs`
  (`[test.web_server]` supervisor)
- Reporting and snapshots: `html_reporter_tests.rs`, `text_snapshot_tests.rs`

## Open gaps

- Side-by-side behavior/reporting comparison against Vitest and Jest on the
  compat corpus fixtures.
