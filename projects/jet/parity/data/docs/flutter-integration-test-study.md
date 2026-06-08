# Flutter integration_test — study (jet parity foundation)

Issue: #2143. Parent epic: #2138. Status: analysis-only deliverable.

## Why this study

The jet parity epic (#2133) needs a perceptual-parity harness that can
drive the *same* fixture through multiple rendering backends (jet's DOM
renderer, jet's WASM renderer, and reference modern-FE stacks) and
compare the resulting DOM / pixels under a per-fixture tolerance budget.
That is structurally the same problem Flutter solved when its web
target shipped three rendering backends (HTML, CanvasKit, Skwasm):
write the test once against a renderer-agnostic widget/finder API, then
have the harness multiplex execution across each backend and surface
per-backend pass/fail with deltas attributed to the renderer.

Flutter's `integration_test` package is the canonical end-to-end answer
in that ecosystem. It is well-trodden, used in CI by the Flutter team
itself, and exposes a clean separation between (a) the in-process
binding that owns the running app, (b) the host-side driver that
launches the binary and collects results, and (c) the rendering target
selected at flutter-drive launch time. That separation maps cleanly
onto jet's planned `jet parity run` topology and is the reason #2143
was scoped as a study before #2144 (the single CI gating manifest spec)
is authored. The findings below feed directly into #2144; the
"Applicable to jet?" column on each pattern row is the contract this
study owes that downstream spec.

This document is analysis-only. No jet code, config, or harness lands
as part of #2143 — the patterns are catalogued here so #2144 can
adopt / adapt / skip them with intent rather than reinventing.

## Architecture

Flutter's `integration_test` package layers four concerns on top of the
existing `flutter_test` widget-test machinery. The layers are visible
in the upstream layout under `packages/integration_test/lib/` in the
`flutter/flutter` repository:

```
packages/integration_test/lib/
  integration_test.dart                  # in-process binding + public API
  integration_test_driver.dart           # host-side driver entrypoint
  integration_test_driver_extended.dart  # extended driver (screenshots, custom callbacks)
  common.dart                            # wire types shared binding <-> driver
  src/                                   # internal channel / response plumbing
```

(File list confirmed via the GitHub contents API against
`flutter/flutter@master`. Individual line counts are not reproduced
here because the package evolves — treat the layout as a stable
contract, the file bodies as moving targets.)

### Layer 1 — In-process binding

`IntegrationTestWidgetsFlutterBinding` (in `integration_test.dart`)
is the linchpin. It is a subclass of the same `LiveTestWidgetsFlutterBinding`
that `flutter_test` uses for "live" tests, but with two crucial
modifications:

1. **It runs against the real running app**, not a synthetic tester
   surface. `pumpWidget(MyApp())` boots the actual production widget
   tree inside the binding's window, so anything the app does at
   runtime (HTTP, plugins, real platform channels) executes for real.
2. **It exposes a results channel** so the host-side driver can read
   per-test pass/fail/skip status and any `reportData` payload the
   test produced. The channel is the `Future<Map<String, dynamic>>`
   that `IntegrationTestWidgetsFlutterBinding.instance.results`
   resolves to once `tearDownAll` runs.

The binding is initialised via
`IntegrationTestWidgetsFlutterBinding.ensureInitialized()` at the top
of the test entrypoint — symmetric with how `flutter_test`'s
`TestWidgetsFlutterBinding.ensureInitialized()` works, which is why an
existing widget test can be migrated into integration_test with near-
zero rewrites.

### Layer 2 — Host-side driver

`integration_test_driver.dart` exposes `integrationDriver()`, a tiny
function that does three things:

1. Connect to the running app's binding over the Dart VM service
   protocol (or, on web, over a WebSocket forwarded by ChromeDriver).
2. Wait for the binding's `results` future to complete.
3. Translate each result into a process exit code for CI.

The driver is the script `flutter drive` runs on the host. Its single
responsibility is *result transport* — it does not know about
renderers, fixtures, or assertions. That is exactly the separation
jet's `jet parity run` orchestrator will need.

`integration_test_driver_extended.dart` is the same idea with an extra
hook: it accepts a `responseDataCallback` so the driver can write
screenshots / golden bytes / arbitrary JSON to disk on the host. The
in-process binding produces the bytes inside the app process; the
extended driver writes them out on the host. This is the canonical
pattern Flutter uses for cross-platform screenshot capture, because
the in-process binding cannot reliably write to host disk on iOS /
Android / web.

### Layer 3 — Wire types

`common.dart` defines the JSON-serialisable types that travel between
the binding and the driver: `Response`, `Failure`, and the test
identifier triple. These are the wire types every renderer must agree
on. (general knowledge: the rough shape is `{ results: Map<String,
String>, failureDetails: List<Failure>, allTestsPassed: bool }`,
serialised as JSON. Confirm in `common.dart` before lifting into a
jet schema.)

### Layer 4 — flutter drive + renderer dispatch

The renderer selection happens entirely at *invocation* time, not in
the test source. On web, the invocation looks like:

```bash
flutter drive \
  --driver=test_driver/integration_test.dart \
  --target=integration_test/app_test.dart \
  -d chrome \
  --web-renderer=canvaskit   # or html, or skwasm
```

`--web-renderer` is a flutter-tool flag that gets propagated into the
compiled JS bundle as a build-time switch. The test source is
identical across the three runs; the rendering stack underneath the
widget tree differs. The CI matrix is therefore "N tests × 3
renderers", with each cell a separate `flutter drive` invocation, and
results aggregated by the host CI script (not by `integration_test`
itself — the package does not own the matrix dimension).

That last point is important and somewhat counterintuitive: Flutter's
`integration_test` does **not** ship a "run this test across all
renderers" loop. It ships a single-renderer execution primitive, and
the matrix is a thin shell script (or GitHub Actions job matrix) on
top. Jet should expect to own the matrix dimension in `jet parity run`
explicitly — it does not come for free from any upstream pattern.

### Web-specific path (HTML / CanvasKit / Skwasm)

On web the architecture has one extra hop. `flutter drive` launches a
local web server that serves the compiled app, then launches
ChromeDriver against that URL. ChromeDriver speaks the WebDriver
protocol; the integration_test driver speaks JSON over a WebSocket
tunnelled through that WebDriver session. The renderer choice
(`html` vs `canvaskit` vs `skwasm`) is fully baked into the served JS
bundle, so the driver layer is renderer-blind — it only sees DOM and
the result envelope coming back over the WebSocket.

Representative upstream code lives in
`packages/integration_test/lib/integration_test.dart`,
`packages/integration_test/lib/integration_test_driver.dart`, and the
flutter-tool side under `packages/flutter_tools/lib/src/web/`
(general knowledge — exact filename has churned). The Flutter team's
own per-renderer CI matrix runs out of `dev/bots/` and the LUCI
recipe configs; specific commit hashes are intentionally not cited
here because they age out fast. If a citation is required for #2144,
re-fetch the current LUCI config and pin the SHA at authoring time.

## Patterns

The table below distils six concrete patterns from the architecture
above, classifies each as `adopt` / `adapt` / `skip` for jet, and (per
R3+R4) names the jet artifact that will carry an adopted pattern, the
deviation for an adapted one, or the architectural reason for a skip.

| Pattern | Flutter mechanism | Applicable to jet? | Rationale |
|---|---|---|---|
| Cross-renderer matrix dispatch | Host shell + `flutter drive --web-renderer=<html\|canvaskit\|skwasm>`, results aggregated by CI not by the in-process binding | adapt | Jet owns the matrix dimension in `jet parity run --backend=dom,wasm,ref-react,ref-svelte`; matrix multiplexing lives in the `jet parity` CLI rather than CI shell so it's testable end-to-end. Section to author in #2144: `## Matrix execution`. |
| Renderer-agnostic finder API | `find.byKey`, `find.byType`, `find.text` resolve against the semantic widget tree, not the rendered pixels | adopt | Jet's hidden-semantics DOM (the `data-jet-*` attributed shadow tree already emitted by the DOM renderer) is the structural analogue. The jet artifact is the `parity::finders` module + the `## Finders` section of #2144's spec — same API surface (`by_key`, `by_role`, `by_text`) over jet's semantic tree. |
| Per-fixture tolerance budget | Not in `integration_test` core; provided by community packages (golden_toolkit, alchemist) that wrap the binding | adapt | Jet ships tolerance as a first-class config key (`tolerance.pixel_diff_pct`, `tolerance.dom_attribute_drift`) in the gating manifest. The deviation: tolerance is per-fixture-per-backend, not global, so the schema is a 2D map keyed by `(fixture_id, backend_id)`. Carried by #2144's `## Tolerance schema` section. |
| Selective per-renderer skip | `testWidgets('...', (t) async { ... }, skip: kIsWeb && renderer == 'html');` — runtime guard inside the test body | adapt | Jet expresses skips declaratively in the gating manifest (`skip: { fixture: 'foo', backend: 'wasm', reason: '#2145', expires: '2026-08-01' }`) rather than as inline `if` guards, so the skip list is grep-able and auditable in CI logs. Carried by #2144's `## Skip / quarantine` section. |
| Screenshot golden capture | `binding.takeScreenshot('name')` in the test + `responseDataCallback` in the extended driver writes bytes to host disk | adopt | Same two-layer split lands in jet: the renderer-side `parity_capture::snapshot(name)` produces a `Snapshot { dom: String, pixels: Option<Vec<u8>> }`, the host-side `jet parity run` writes goldens to `projects/jet/data/parity/goldens/<backend>/<fixture>.{html,png}`. Carried by #2144's `## Golden capture` + the `jet parity capture` CLI verb. |
| In-process binding owns lifecycle | `IntegrationTestWidgetsFlutterBinding.ensureInitialized()` is the singleton; the host driver is a thin transport | skip | Jet's parity runs are headless and out-of-process by default — the renderer is invoked as a child process from `jet parity run`, with results returned via stdout JSON, not via a long-lived binding singleton. The binding model assumes a hot-reloadable Dart VM, which jet (Rust + WASM) does not have. |

Notes:

- The "matrix dispatch" row is the single most important pattern in
  the table. It's the architectural inversion that makes Flutter's
  test source renderer-blind: the test never asks "which renderer am
  I on?", the *runner* asks "which row of the matrix am I?". Jet
  should preserve that inversion at all costs.
- The "finder API" row is the second most important. Without a
  renderer-agnostic finder layer, every assertion ends up coupled to
  whichever backend was used to author the fixture, and parity
  regressions in the *other* backends become invisible.
- The "tolerance" row diverges from Flutter because Flutter's core
  package punted on tolerance and let the ecosystem solve it. Jet
  cannot punt — perceptual parity *is* tolerance-shaped, so it lands
  in the gating manifest as a first-class field.

## Non-goals

The following capabilities exist in Flutter's `integration_test` (or in
its surrounding ecosystem) and are explicitly **out of scope** for
jet's parity foundation. They are listed here so #2144 can cite this
section by anchor rather than re-deriving the boundary.

- **Mobile device farms.** `flutter drive` integrates with Firebase
  Test Lab to fan out a single test across hundreds of physical
  Android / iOS devices. Jet's parity surface is the DOM, not native
  mobile UI — the renderer is the dimension, not the device. No
  device-farm integration is in scope for #2144.
- **Native iOS / Android testbench.** The `android/` and `ios/`
  directories in `packages/integration_test/` wire the binding into
  native test runners (JUnit / XCTest). Jet has no native mobile
  testbench and no plan to acquire one in the parity epic.
- **Screen-reader as a first-class channel.** `integration_test` can
  drive Flutter's `SemanticsBinding` to assert against the
  accessibility tree, and there's a long-running thread about
  exercising real platform screen readers. Jet's hidden-semantics DOM
  is already auditable directly; a real-screen-reader pass would be a
  separate epic, not part of parity foundation.
- **Flutter-specific golden-file viewer UI.** Tools like
  `golden_toolkit`'s HTML report and `alchemist`'s side-by-side diff
  UI are productive but couple tightly to Dart tooling. Jet will emit
  goldens in a renderer-neutral on-disk layout and lean on a
  static-site diff viewer (out of #2144 scope; tracked in #2138's
  follow-up backlog).
- **Live reload / hot restart during a test.** `IntegrationTestWidgetsFlutterBinding`
  supports running multiple `testWidgets` against a single hot-restarted
  app instance. Jet's per-process model re-launches the renderer per
  fixture for hermetic isolation; hot-restart parity is not in scope.
- **patrol-style native dialog interaction.** The community `patrol`
  package extends `integration_test` to drive native permission
  dialogs and notification trays. Jet has no native chrome to drive —
  out of scope.
- **Performance profiling via DevTools timeline.** `integration_test`
  exposes `traceAction` for capturing timeline events. Jet may want
  perf parity later (it would be a separate epic against #2133), but
  it is not part of the perceptual-parity foundation.
- **Per-test seed scrubbing for flake hunts.** Flutter's CI runs each
  integration test multiple times with different random seeds to
  surface flake. Jet's fixtures are deterministic by construction
  (jet renders are pure functions of fixture state), so seed-scrubbing
  is unnecessary at this layer.

## Open questions

These are decisions that #2143 deliberately does not resolve. Each is
sized for a follow-up `aw wi` once #2144 begins authoring.

- **Where does jet's matrix live — CLI or CI?** Flutter put it in CI
  (host shell loops over `flutter drive` invocations). Jet's draft
  put it in the `jet parity run` CLI. Trade-off: CLI is more testable
  and reproducible locally; CI is simpler and lets each renderer fail
  independently in the GitHub Actions UI. Likely answer: CLI owns the
  matrix dimension, CI invokes the CLI once per push with a
  `--matrix=all` flag.
- **Per-fixture tolerance or per-fixture-per-backend tolerance?**
  The table above lands on 2D (fixture × backend). Open question:
  do we also need per-region tolerance for fixtures that have a known
  noisy region (e.g. a glyph atlas corner)? Candidate answer space:
  start 2D in #2144, expand to per-region with a `regions: [{rect, tol}]`
  sub-schema only if a real fixture motivates it.
- **Golden storage — in-repo bytes or content-addressed sidecar?**
  Flutter checks PNGs into the repo (with LFS for size). Jet's
  goldens may be larger (DOM snapshots + pixels) and more frequently
  rotated. Candidate answer space: small DOM goldens in-repo, large
  pixel goldens behind a CAS lookup pinned by SHA in the gating
  manifest.
- **Skip-expiry enforcement — soft warning or hard fail?** The skip
  manifest entry in §Patterns has an `expires` field. Open: when an
  expiry passes and the skip is still present, does CI hard-fail or
  emit a warning? Candidate answer space: hard-fail in CI, with a
  one-week grace window surfaced via a separate warning job, mirroring
  Rust's `#[deprecated]` posture.
- **Finder API parity surface — minimum viable set?** Flutter's
  `find.*` exposes ~15 finders. Which subset does jet need on day
  one? Candidate answer space: `by_key`, `by_role`, `by_text`,
  `by_test_id`, `by_predicate` — five finders cover the bulk of real
  assertions, with the rest deferred to a "finders v2" issue.
- **Result transport — JSON-over-stdout or a structured channel?**
  Flutter uses a Dart-VM-protocol channel under the hood, which gives
  it streaming + back-pressure. Jet's headless renderers do not have
  an equivalent VM-protocol surface and would default to one large
  JSON blob over stdout at process exit. Candidate answer space:
  stdout JSON blob is fine for v1 (fixtures are bounded); if a single
  fixture grows past ~10 MB of golden bytes, revisit with a sidecar
  file referenced by hash from the stdout envelope.

## Appendix A — concrete mapping table for #2144

The following table is the contract between this study and #2144's
authoring step. Each row in §Patterns above with an `adopt` or
`adapt` verdict is reproduced here with the explicit section heading
that #2144 must contain. The reviewer of #2144 should use this table
as the structural checklist when validating the spec.

| Pattern (from §Patterns) | #2144 section heading | Section type |
|---|---|---|
| Cross-renderer matrix dispatch | `## Matrix execution` | logic |
| Renderer-agnostic finder API | `## Finders` | interfaces |
| Per-fixture tolerance budget | `## Tolerance schema` | interfaces |
| Selective per-renderer skip | `## Skip / quarantine` | interfaces |
| Screenshot golden capture | `## Golden capture` + `jet parity capture` CLI verb | interfaces + cli |

The `In-process binding owns lifecycle` row (verdict: skip) does
**not** appear in #2144 because the architectural decision is to
not adopt it; the rationale in §Patterns is sufficient documentation.

## Appendix B — terminology crosswalk

Because future readers of #2144 will already be steeped in jet's
vocabulary, the following crosswalk maps Flutter terms used in this
study to their jet analogues. This is purely a glossary; it does not
constrain #2144's wording.

| Flutter term | jet analogue | Notes |
|---|---|---|
| widget tree | semantic DOM (`data-jet-*` attributed) | jet's tree is HTML-shaped; Flutter's is Dart object graph |
| `WidgetTester` | `parity::Driver` (planned) | the per-fixture handle that owns finders + asserts |
| `pumpAndSettle()` | `driver.settle()` (planned) | waits until jet renderer reports no pending frames |
| `find.*` | `parity::finders::*` (planned) | renderer-agnostic locators |
| `testWidgets(...)` | a `#[parity_fixture]` Rust attribute (planned) | per-fixture entrypoint |
| host-side `flutter drive` script | `jet parity run` CLI | invokes child renderer, collects results |
| `--web-renderer` flag | `--backend` flag on `jet parity run` | selects which renderer the fixture exercises |
| golden file (PNG under repo) | `projects/jet/data/parity/goldens/<backend>/<fixture>.{html,png}` | jet stores both DOM and pixel goldens |
| `responseDataCallback` | `jet parity run --capture` flag | writes captured bytes to host disk |

## References

- `packages/integration_test/lib/integration_test.dart` — in-process
  binding (`flutter/flutter` repo, master).
- `packages/integration_test/lib/integration_test_driver.dart` and
  `integration_test_driver_extended.dart` — host-side drivers, same
  repo.
- `packages/integration_test/lib/common.dart` — wire types.
- Flutter docs: `https://docs.flutter.dev/testing/integration-tests`
  and the cookbook at
  `https://docs.flutter.dev/cookbook/testing/integration/introduction`.
- Consumer of this study: #2144 (parity foundation — single CI gating
  manifest spec). Every "adopt" / "adapt" row above must surface as a
  named section in that spec.
