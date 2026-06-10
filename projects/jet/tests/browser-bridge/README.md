# Block: browser bridge — replace Playwright (CLI + MCP)

**Claim.** `jet bb` fully replaces Playwright for browser automation and e2e:
agent-first detached/headless control, Playwright-parity Page/Locator/expect
APIs, route interception, storage state, screenshots, trace/evidence
artifacts — exposed both as a CLI (`jet bb ...`) and as an MCP server so
agents can drive it without shelling out.

**Replacement rule.** Runtime gates use `jet bb`/`jet browser`; Playwright is
the comparison target, never the executor.

## Gates

| Gate | Command | Covers |
|---|---|---|
| Browser lib suite | `cargo test -p jet --lib browser -- --nocapture` | CDP driver, launcher, context |
| CLI smoke | `cargo test -p jet --test browser_cli_smoke` | `jet browser` end-to-end debugging commands |
| Playwright baseline phase | `JET_BASIC_DOM_BROWSER_BASELINES=playwright JET_BASIC_DOM_REQUIRE_BROWSER_BASELINES=1 projects/jet/scripts/verify-basic-dom-gates.sh --phase browser` | replacement evidence vs a Playwright baseline |
| Replacement report | `node projects/jet/scripts/verify-browser-bridge-replacement.mjs` | command-surface replacement checks |

## In this folder

- Runtime/CLI: `browser_cli_smoke.rs`, `browser_context.rs`,
  `browser_install.rs`
- MCP server: `bb_mcp_server.rs` — `jet bb mcp` serves the Browser Bridge
  over MCP stdio (initialize/tools-list/tools-call contract; tool failures
  must be `isError`, never protocol errors or stdout noise)
- Playwright API parity: `page_api_parity.rs` (R1–R27), `locator_js_api.rs`,
  `matchers_state_value_a11y.rs`, `to_have_screenshot_tests.rs`,
  `route_intercept_tests.rs`, `storage_state_tests.rs`,
  `page_fixture_auto_inject.rs`
- `@playwright/test` compat shims: `playwright_compat_tests.rs`,
  `playwright_compat_shim_tests.rs`, `e2e_playwright_residue.rs` (no
  Playwright dependency may survive in the e2e harness)
- Trace and evidence: `trace_capture.rs`, `trace_viewer.rs`,
  `auto_artifacts_tests.rs`, `product_step_timeline.rs`,
  `pm_report_acceptance.rs`, `pm_report_static_smoke.rs`,
  `cue_artifact_studio_dogfood.rs`

Compat-corpus fixtures: `../fixtures/playwright-compat/`.

## Open gaps before "full replacement" is claimable

- MCP surface breadth: `jet bb mcp` (src/browser_cli/mcp.rs) exposes
  launch/shutdown/tree/hooks/eval/capture/screenshot/mouse/drag/wheel/key/
  highlight today; locator-level tools (query/click-by-selector with
  auto-waiting) and trace tools are not exposed over MCP yet.
- Locator/actionability parity breadth vs Playwright (auto-waiting matrix,
  strictness violations, iframe/shadow DOM coverage).
- Trace replay hooks and richer failure diagnostics.
- WASM browser-API bridge coverage (shared with the `wasm/` block).
