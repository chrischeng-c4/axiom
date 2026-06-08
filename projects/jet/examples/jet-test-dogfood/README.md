# jet-test-dogfood

Minimal dogfood corpus for `jet test`. Proves the runner is usable
without a published `@jet/test` package — every spec imports from the
in-tree `@jet/test` virtual module and runs through the native runner.

The corpus covers three shapes:

- `src/unit.spec.ts` — pure-logic unit tests (math, string formatting,
  pure functions). No DOM, no fetch, no fixtures.
- `src/frontend-integration.spec.ts` — integration-style tests that
  wire several pure modules together (a tiny todo store reducer +
  selectors). Still `env=node`, still no DOM.
- `src/failure-fixture.spec.ts` — intentionally failing test that
  produces structured agent-readable result data: `expect.toBe` diff
  + stack location + rerun hint.

Run from the repo root:

```bash
cargo run -p jet -- test projects/jet/examples/jet-test-dogfood/src --reporter=json
```

The JSON reporter writes `.jet/test-results.json` carrying
`schema_version: "jet.test.result.v1"` and a `reports[]` entry per test.
Failing tests carry `error.message`, `error.diff`, and `error.stack`,
which is the contract agents read.

Per #2606 this corpus stays inside the `node` test environment — DOM /
component / product-flow shapes live behind `--env=dom`,
`--env=component`, and `jet e2e` respectively.
