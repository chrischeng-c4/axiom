# Service Http Capabilities

## Brief

Four services in this repository — lumen, keep, relay, loom — answer the same
questions before they answer a single business request. Which routes must stay
reachable when the data plane requires a token? What does a request that
arrives with someone else's `traceparent` become? What shape is an error body?
How large may a request body get, and what does the refusal look like? How
many requests per second may one caller make, and how is that caller
remembered without remembering the caller?

`service-http` owns those answers, once. It is the HTTP policy shell an
adopting service composes: the five standard probe routes, the accept-or-
generate W3C trace context and the access log derived from it, the
`{"error", "message"}` envelope, the request-body byte cap, opaque-key
admission control with its environment grammar, and per-response
`Server-Timing` attribution.

It composes rather than replaces. Listener ownership belongs to `server-http`;
drain and readiness to `server-lifecycle`; log/trace/metric provider ownership
to `service-observability`. This crate re-exports those seams under HTTP-facing
names and adds no policy of its own to them. Auth is deliberately not here —
`service-auth` owns it, and the one place that boundary is visible in this
crate's behavior (the `Server-Timing` disclosure default) is documented as a
decision rather than a gap.

## Capabilities

Every capability belongs to exactly one of two feature roots:

- **Core Features** are the shell itself: what a request meets on the way in
  (probe routing, trace context, body cap) and what an error looks like on the
  way out.
- **Non-Core Features** are the opt-in policies a service layers onto that
  shell — admission control, its configuration grammar, timing disclosure, and
  the access log's probe demotion. Non-core does not mean optional; it means
  the adopting service decides whether to wire it.

This file contains stable product promises, claim IDs, and verification
surfaces. Delivery planning lives outside this contract and references these
IDs one way.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Standard Probe Surface | 3377 | implemented | verified | smoke | ready | core; five auth-exempt, body-limit-exempt routes every k8s-native service ships, with `/readyz` reporting 503 the moment a drain begins |
| Accept-or-Generate Trace Context | 3377 | implemented | verified | smoke | ready | core; a strictly valid W3C version-00 `traceparent` is preserved, anything else is treated as absent and a fresh local root is minted instead of rejecting the request |
| Shared Error Envelope | 3377 | implemented | verified | smoke | ready | core; one `{"error", "message"}` JSON body with a fixed field order, rendered from a status plus a machine-stable kind |
| Request Body Byte Cap | 3377 | implemented | verified | smoke | ready | core; a declared oversized body is refused before a byte is read and a streamed one is bounded mid-read, both rendering the crate's own 413 envelope |
| Opaque-Key Admission Control | 3377 | implemented | verified | smoke | ready | non-core; per-class token buckets whose retained state is a SHA-256 fingerprint and whose key ledger is bounded by eviction, denying with 429 and a `Retry-After` derived from the actual deficit |
| Opt-In Admission Configuration | 3377 | implemented | verified | smoke | ready | non-core; one `<PREFIX>_ADMISSION_*` grammar shared by every adopter, disabled unless a class capacity is set, and refusing a tuning knob that would silently do nothing |
| Server-Timing Response Attribution | 3377 | implemented | verified | smoke | ready | non-core; an always-present `app;dur=` baseline plus handler-pushed phases that render only on a response explicitly marked full-disclosure |
| Probe-Demoted Access Log | 3377 | implemented | verified | smoke | ready | non-core; one access event per response at INFO for the data plane and DEBUG for the five probe paths, so a scrape loop cannot drown the log a human reads |

### Core Features

#### Lifecycle-owned HTTP serving

Production services use one `LifecycleController` for probe evidence, listener
admission, and the shared shutdown deadline via `serve_with_lifecycle` and
`LifecycleShutdownTrigger`; legacy readiness and drain helpers remain migration
adapters.

#### Standard Probe Surface

ID: standard-probe-surface
Root WI: 3377
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke

Promise:
`standard_probe_routes` mounts the five always-on routes every k8s-native
service in this repository ships on its one serve port: `GET /healthz`
(liveness, 200 as long as the process can answer), `GET /readyz` (200 `ok`, or
503 `draining` once the service's `ReadinessHook` reports a drain in progress),
`GET /metrics` (Prometheus text format, `text/plain; version=0.0.4`), `GET
/openapi.json`, and `GET /docs` (a Swagger UI page pointed at this pod's own
`/openapi.json`). The router carries **no auth layer and no body limit** —
that is the point of it existing separately from the data plane. A kubelet
probe and a Prometheus scrape must reach a service that otherwise requires a
token, and a service merges its own auth'd, body-limited routes onto this
router rather than the other way round.

Two decisions are load-bearing. `/readyz` reads the drain flag on every hit
rather than caching it, so the 503 appears at the start of the grace window
and k8s stops routing before the listener closes. And `/metrics` with no
provider serves an empty 200 rather than a 404 — a service without metrics is
still a scrapeable service, and a 404 would show up in a scrape dashboard as a
broken target.

`standard_probe_routes_canonical_json` is the byte-identical variant: instead
of serializing a typed `utoipa` document per request it returns the producer's
exact bytes, so a service that must prove its committed client snapshot, its
offline CLI, and its live route are one document can do so.

Surfaces:
- Rust API: `service_http::standard_probe_routes` - the five standard routes over a typed OpenAPI document.
- Rust API: `service_http::standard_probe_routes_canonical_json` - the same routes with a byte-exact OpenAPI producer.
- Rust API: `service_http::ReadinessHook` - the drain seam `/readyz` consults.
- Rust API: `service_http::MetricsProvider` - the Prometheus text seam `/metrics` renders.

Rust internal: the shared `ProbeState` both constructors build, and the `OpenapiSource` enum that keeps the two variants one router rather than two.

EC Dimensions:
- behavior: `cargo test -p service-http --lib` - each of the five routes answers with its documented status and body, including the empty-provider `/metrics` case and the canonical-JSON variant returning the producer's exact bytes.
- security: `cargo test -p service-http --lib` - `/readyz` returns 503 `draining` when the readiness hook reports a drain, so a draining pod cannot keep receiving traffic by reporting ready.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Five routes answer as documented | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; `probes::tests::healthz_is_ok`, `probes::tests::openapi_json_parses` and `probes::tests::docs_serves_swagger_page` assert status and body per route, so the surface is observed rather than described |
| Drain flips readiness to 503 | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; `probes::tests::readyz_503_when_draining` and `probes::tests::readyz_200_when_not_draining` prove both directions, so a one-sided assertion cannot pass a hook that is always ready |
| Absent metrics provider is not an error | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; `probes::tests::metrics_empty_when_no_provider` pins the 200-with-empty-body case against `probes::tests::metrics_renders_provider_text`, so a scrape target without metrics does not read as broken |
| Canonical JSON keeps producer bytes | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; `probes::tests::canonical_openapi_keeps_producer_bytes` asserts byte equality with the producer, so a re-serialization cannot silently change the document a snapshot pinned |

#### Accept-or-Generate Trace Context

ID: accept-or-generate-trace-context
Root WI: 3377
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke

Promise:
`request_trace_context` reads the inbound `traceparent` header and always
returns a usable correlation context. When the header is a strictly valid W3C
version-00 value, its `trace_id` and its span id (which becomes this request's
`parent_span_id`) and its `trace_flags` are preserved, and a fresh `span_id`
is minted for this hop. When it is absent, or invalid in any way, the result is
deliberately identical to no input at all: a fresh local-root `trace_id`, a
fresh `span_id`, no parent, and `trace_flags` of `00`. **Invalid input never
fails the request.** A caller's malformed header is a correlation problem, not
an authorization problem, and refusing business traffic over it would let an
upstream typo take a service down.

Strict means strict: exactly 55 characters, hyphens at exactly the three
positions the format fixes, version exactly `00`, every id character a
lowercase hex digit — uppercase is rejected rather than normalized — and
neither the trace id nor the parent span id all zeros, since the W3C format
reserves those as the invalid value. More than one `traceparent` header is
also rejected, because there is no defensible rule for choosing between two
claimed parents. A zero `trace_flags` value is *not* rejected: `00` is the
ordinary "not sampled" flag byte and is the same value a fresh root gets.

`CorrelatingMakeSpan` records `trace_id`, `span_id`, `trace_flags` and — only
when one exists — `parent_span_id` onto every request span, so cross-service
log correlation works in a build with no exporter configured at all. Under the
`otlp` feature the same span additionally adopts the extracted OpenTelemetry
parent context, without changing the accept-or-generate contract above.

Surfaces:
- Rust API: `service_http::transport::request_trace_context` - the accept-or-generate parse.
- Rust API: `service_http::transport::RequestTraceContext` - the resolved correlation fields.
- Rust API: `service_http::trace_layer` - the composed tracing layer an adopter attaches to its outer router.
- Rust API: `service_http::PropagatingMakeSpan` - the span maker that attaches a valid propagated parent under the `otlp` feature.

Rust internal: the fixed-offset `traceparent` parser with its lowercase-hex and all-zero rejections, and the fresh-id generator that will not return an all-zero id.

EC Dimensions:
- behavior: `cargo test -p service-http --test request_trace_context` - a valid inbound `traceparent` yields a context whose trace id matches the header and whose parent span id is the header's span id, while a fresh span id is minted for this hop.
- security: `cargo test -p service-http --test request_trace_context` - a malformed, wrong-version, or all-zero `traceparent` produces a safe local root and the request still routes, so a hostile or broken upstream header cannot deny service or forge a trace lineage.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Valid parent is preserved | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --test request_trace_context`; `valid_traceparent_preserves_trace_and_creates_child_span` asserts both the preserved trace id and that the span id differs from the parent's, so a pass-through that forgot to mint a child would fail |
| Invalid input degrades, never rejects | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --test request_trace_context`; `invalid_or_missing_traceparent_creates_safe_root` proves the request still routes and the context is a fresh root, so strictness cannot turn into a denial-of-service surface |
| Correlation works with no exporter | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --test request_trace_context`; `trace_layer_records_context_and_routes_without_otlp` runs the layer in a logging-only build, so the correlation claim does not secretly depend on OTLP being wired |

#### Shared Error Envelope

ID: shared-error-envelope
Root WI: 3377
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke

Promise:
Every error response an ecosystem service renders is the same two-field JSON
body: `error`, a short machine-stable classification a client may branch on,
and `message`, human-readable detail a client should not parse. `ApiErr` pairs
that body with a `StatusCode` and renders it through `IntoResponse`, so a
service writes one `From<DomainError>` arm per classification and never
hand-builds a response. The field order is fixed — `error` before `message` —
because a client that byte-compares a fixture should not break when an
unrelated field is added elsewhere in the crate. The envelope also round-trips
through `Deserialize`, so a caller in this workspace can consume it as a typed
value rather than by string matching.

This crate owns the envelope, the builder, and the `utoipa::ToSchema` shape —
never the domain classification, which stays in the service that has the
domain. It is also the shape this crate's own refusals use: the admission
429 and the body-limit 413 both render through `ApiErr`, so an adopting
service's error contract has no exceptions carved out for the shell's own
rejections.

Surfaces:
- Rust API: `service_http::ErrorEnvelope` - the `{error, message}` wire shape.
- Rust API: `service_http::ApiErr` / `ApiErr::new` - the status + kind + message builder.

Rust internal: the `IntoResponse` implementation that is the single rendering path for the envelope, shared by this crate's own 413 and 429.

EC Dimensions:
- behavior: `cargo test -p service-http --lib` - a built `ApiErr` renders the intended status with `error` and `message` carrying the supplied kind and text, and the envelope deserializes back to the same values.
- security: `cargo test -p service-http --lib` - the serialized field order is pinned to `error` then `message`, so a client contract or fixture cannot be broken by a silent reordering.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Status and body render together | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; `error::tests::renders_status_and_envelope_json` asserts the status alongside both body fields, so a correct body with the wrong status would still fail |
| Field order is pinned | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; `error::tests::envelope_serializes_error_before_message` compares the exact serialized string rather than a parsed value, so ordering is actually constrained |
| Envelope round-trips | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; `error::tests::envelope_round_trips_through_deserialize` proves the type is consumable, not just producible |

#### Request Body Byte Cap

ID: request-body-byte-cap
Root WI: 3377
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke

Promise:
`body_limit_layer(max_bytes)` is the one place a service's `body_limit_bytes`
configuration is actually enforced. It bounds a request body two ways, because
one way is not enough. A request that *declares* a `Content-Length` over the
cap is refused immediately, before the body is read at all — the cheap case,
and the one an honest client hits. A request that declares nothing, or
under-declares, is still bounded: its body is wrapped so a stream that grows
past the cap errors mid-read instead of being buffered without limit. A
chunked body never carries a trustworthy length, so treating a missing or
unparseable `Content-Length` as "not oversized" is correct only because the
wrapped body is the real enforcement.

The threshold is strictly over, not at: a body of exactly `max_bytes` passes.
Every 413 the layer is responsible for — whether short-circuited on the header
or produced downstream once the wrapped body errors during extraction — is
rewritten into this crate's `{"error": "payload_too_large", "message": ...}`
envelope, rather than the plain-text rejection the extractor would otherwise
emit. That rewrite is what makes the error contract uniform; without it a
service's largest-request failure would be the one response in its API that
is not JSON.

Surfaces:
- Rust API: `service_http::body_limit_layer` - build the cap layer for a byte count.
- Rust API: `service_http::BodyLimitLayer` / `service_http::BodyLimitService` - the tower layer and service types.
- Rust API: `service_http::HttpConfig::body_limit_bytes` - the configured cap an adopter passes in.

Rust internal: the `Content-Length` pre-check with its strict-over comparison, and the downstream 413 rewrite that catches the streaming case.

EC Dimensions:
- behavior: `cargo test -p service-http --test body_limit` - a body under the cap and a body exactly at the cap both pass through, and a streamed body with no `Content-Length` under the cap also passes.
- security: `cargo test -p service-http --test body_limit` - a declared-oversized body is refused with the structured 413, and a streamed body with no `Content-Length` that grows past the cap is also refused, so the header path cannot be bypassed by omitting the header.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Declared oversize refused with structured 413 | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --test body_limit`; `content_length_known_oversized_body_is_rejected_with_structured_413` asserts both the status and the envelope shape, so a plain-text 413 would fail |
| Header omission does not bypass the cap | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --test body_limit`; `streaming_body_without_content_length_over_cap_is_rejected` paired with `streaming_body_without_content_length_under_cap_passes_through` proves the streaming path both rejects and admits, so a layer that rejected everything would fail |
| Cap is strictly over, not at | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib --test body_limit`; `body_limit::tests::content_length_exceeds_is_strict_over_not_at_the_cap` and `at_limit_body_passes_through` pin the boundary at both the unit and the router level |
| Absent or garbage header is not oversized | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; `body_limit::tests::content_length_exceeds_is_false_when_header_absent_or_unparseable` proves the pre-check defers to the wrapped body rather than guessing |

### Non-Core Features

#### Opaque-Key Admission Control

ID: opaque-key-admission-control
Root WI: 3377
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke

Promise:
`AdmissionController` gives a service per-endpoint-class token buckets keyed on
a caller-owned opaque key — a token, a principal id, a client address,
whatever the service decides identifies "one caller". Three properties make it
safe to hand a credential to.

**The key is never retained.** `AdmissionInput` is deliberately neither `Debug`
nor serializable, and exists only until `admit` hashes it; what the bucket map
holds is a SHA-256 fingerprint. The observer hook sees the class, the outcome,
and the retry-after milliseconds — never the key, never the fingerprint — so
wiring admission to a metrics or audit sink cannot leak a credential into a
log aggregator.

**The state is bounded.** Each class holds at most `max_keys` buckets; a new
key arriving at a full class evicts the least recently used one in that class
first. An unbounded ledger keyed on attacker-supplied input is a memory
exhaustion primitive, and this is why `max_keys` is a required constructor
argument rather than an option with a generous default.

**The decision is deterministic and the wait is real.** Credits refill
continuously against an explicit clock reading — `admit_at` takes the elapsed
duration, so a test or simulation drives it without sleeping — and a denial's
`Retry-After` is computed from the actual credit deficit rather than a fixed
back-off. A class with no configured policy bypasses without allocating any
state at all, so enabling admission on two routes does not silently start
tracking every route.

`admission_middleware` turns a denial into the standard refusal: HTTP 429, the
crate's `{"error": "rate_limited", ...}` envelope, and a `Retry-After` header
in whole seconds, rounded up and never below 1 — a `Retry-After: 0` would
invite an immediate retry into a limit that has not yet refilled.

Surfaces:
- Rust API: `service_http::AdmissionController` / `AdmissionController::admit` / `admit_at` - the decision, with a clock seam.
- Rust API: `service_http::AdmissionPolicy` / `AdmissionPolicy::new` - capacity, refill window, and key bound, validated at construction.
- Rust API: `service_http::AdmissionInput` - the class plus opaque key, intentionally not printable.
- Rust API: `service_http::AdmissionObserver` / `AdmissionEvent` / `AdmissionOutcome` - the key-free observation hook.
- Rust API: `service_http::admission_middleware` / `service_http::AdmissionMiddleware` - the axum wiring with the service-owned classifier.

Rust internal: the nanosecond-credit token bucket, the per-class LRU eviction that runs before insertion, and the deficit-derived retry-after computation.

EC Dimensions:
- behavior: `cargo test -p service-http --lib` - a capacity-2 class allows twice and denies the third request at the same instant, reports the exact refill wait, and admits again once that wait has elapsed; an unconfigured class bypasses.
- security: `cargo test -p service-http --lib` - the serialized observer events contain none of the three distinct raw keys fed in, nor the words `fingerprint` or `credential`, and a class configured for 2 keys holds exactly 2 after 3 distinct keys arrive, so neither credential leakage nor unbounded growth is possible.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Deterministic allow, deny, and refill | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; `admission::tests::allow_deny_and_refill_are_deterministic` asserts the exact `Duration::from_secs(5)` retry-after and the subsequent allow at that instant, so an arbitrary back-off constant would fail |
| Observer schema carries no key material | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; `admission::tests::state_is_bounded_and_observer_schema_is_key_free` serializes the recorded events and asserts each raw key is absent, so redaction is proven on the serialized bytes rather than assumed from the type |
| Key ledger is bounded by eviction | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; the same test asserts `tracked_keys("write") == 2` after three distinct keys, so an unbounded map would fail |
| Unconfigured class allocates nothing | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; `admission::tests::unconfigured_class_bypasses_without_allocating_state` asserts both the `Bypass` outcome and a zero key count, so a bypass that still allocated would fail |
| Denial renders 429 with Retry-After | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; `admission::tests::middleware_returns_standard_429_and_retry_after` drives a live router and pins the header to `10` and the body's `error` to `rate_limited` |

#### Opt-In Admission Configuration

ID: opt-in-admission-configuration
Root WI: 3377
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke

Promise:
Every adopting service configures admission the same way, under its own
prefix: `<PREFIX>_ADMISSION_READ_CAPACITY`, `_WRITE_CAPACITY`,
`_ADMIN_CAPACITY`, `_REFILL_SECS`, `_MAX_KEYS`. `AdmissionConfig::from_lookup`
owns that grammar; the service keeps its own public route-class names and
supplies them when building the controller, so the shared parser never has to
know that lumen calls a class `lumen.read`.

Admission is off unless at least one class capacity is set. The refill window
defaults to 60 seconds and the key bound to 1024, and both are refused when
they appear *without* any capacity: setting `_REFILL_SECS` on a service with
no capacities configured is an operator who believes rate limiting is on when
it is not, and a silent no-op there is worse than an error at startup. Every
value must parse as a positive integer — a zero capacity would admit nothing,
a zero window would divide by nothing, a zero key bound would hold nothing, so
all three are rejected at parse rather than surfacing later as a policy
construction panic. Parsing goes through an injected lookup rather than reading
the process environment directly, so a test or an embedding caller configures
it without mutating global state.

Surfaces:
- Rust API: `service_http::AdmissionConfig::from_env` - parse from the process environment under a prefix.
- Rust API: `service_http::AdmissionConfig::from_lookup` - the same grammar over an injected lookup seam.
- Rust API: `service_http::AdmissionConfig::is_enabled` / `controller` - the opt-in check and the class-naming construction.
- Rust API: `service_http::AdmissionConfigError` - invalid value, orphaned common setting, and invalid policy.
- Rust API: `service_http::AdmissionConfig::DEFAULT_REFILL_SECS` / `DEFAULT_MAX_KEYS` - the documented defaults.

Rust internal: the shared suffix key builder, the positive-integer filter both capacities and common settings pass through, and the pre-flight policy validation that runs before any controller is built.

EC Dimensions:
- behavior: `cargo test -p service-http --lib` - an empty environment yields a disabled config that builds no controller, and a fully populated one builds a multi-class controller whose buckets enforce the configured capacity.
- security: `cargo test -p service-http --lib` - a non-numeric capacity and a `_REFILL_SECS` set with no capacity are both refused with the offending key named in the message, so a misconfigured limiter fails loudly instead of running open.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Disabled without a capacity | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; `admission::config_tests::config_without_capacities_is_disabled` asserts both `is_enabled()` false and that `controller(..)` returns `None`, so a config that reported disabled while still building a limiter would fail |
| Configured classes actually enforce | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; `admission::config_tests::config_builds_multi_class_controller` drives the built controller to a denial and asserts the tracked key count, so the parse is proven to reach real buckets |
| Invalid and orphaned settings refused | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --lib`; `admission::config_tests::config_rejects_invalid_or_orphaned_common_values` asserts the offending key appears in each message, so an operator gets the key to fix rather than a generic parse error |

#### Server-Timing Response Attribution

ID: server-timing-response-attribution
Root WI: 3377
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke

Promise:
`server_timing_middleware` puts a W3C `Server-Timing: app;dur=<ms>` header on
every response it wraps, measured across the same request/response boundary
`trace_layer` spans — an integrator with no access to Prometheus or to the
service's logs still gets per-request latency from the response itself.
Durations render with three decimal places, so a sub-millisecond phase shows
as a nonzero number rather than `0`.

Phase attribution is opt-in per response, and defaulted closed. Handlers push
named durations onto the per-request `ServerTimingExt` extension, but those
entries render only on a response that explicitly carries
`ServerTimingDisclosure::Full`; every response without that marker — which is
every response, until a service sets one — is `TotalOnly`. That default is a
decision, not an omission: this crate does not depend on `service-auth` and
there is no crate-neutral marker for "this request authenticated", so it
cannot gate a phase breakdown on auth state and refuses to guess. A service
that can see its own auth outcome opts a response in itself.

A phase name is sanitized rather than dropped: anything outside ASCII
alphanumerics, `_`, `-` and `.` becomes `_`, and a name that was empty to
begin with renders as `phase`. A bad name degrades the header; it never loses
the measurement and never fails the request.

Surfaces:
- Rust API: `service_http::server_timing_middleware` - the layer an adopter adds explicitly.
- Rust API: `service_http::ServerTimingExt` / `ServerTimingExt::push` - the per-request phase collector.
- Rust API: `service_http::ServerTimingDisclosure` - the per-response `TotalOnly` / `Full` decision.

Rust internal: the three-decimal millisecond rendering, the token sanitizer with its empty-name fallback, and the drain that leaves the collector empty after a full-disclosure render.

EC Dimensions:
- behavior: `cargo test -p service-http --test server_timing` - a live router's response carries a parseable `Server-Timing` header, and under full disclosure the phases render after the baseline in push order.
- security: `cargo test -p service-http --test server_timing` - the default posture hides pushed phases, and a phase name carrying header-structural bytes is sanitized rather than emitted, so a handler-supplied name cannot inject a delimiter into the header.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Baseline is present and parseable | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --test server_timing`; `header_is_present_and_parseable_on_a_live_router` asserts against a real response rather than the render helper, so a middleware that computed the value but never attached it would fail |
| Default posture hides phases | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --test server_timing`; `default_posture_hides_pushed_phases` paired with `full_disclosure_reveals_phases_in_push_order_after_baseline` proves both postures, so a middleware that always hid — or always revealed — would fail one of them |
| Phase names cannot inject delimiters | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --test server_timing`; `disallowed_phase_name_bytes_are_sanitized_not_dropped` and `server_timing::tests::sanitize_token_replaces_disallowed_bytes_and_falls_back` pin `;` and `,` replacement plus the empty-name fallback |
| Collector is per request | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --test server_timing`; `phase_append_extension_is_per_request_not_shared_across_calls` proves one request's phases cannot appear on another's response |

#### Probe-Demoted Access Log

ID: probe-demoted-access-log
Root WI: 3377
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke

Promise:
`trace_layer` emits one access event per response at the `http.access` target,
carrying the response status and the measured latency in milliseconds. Data
plane requests log at INFO, so the default `info` filter keeps them. The five
probe paths — `/healthz`, `/readyz`, `/metrics`, `/openapi.json`, `/docs` —
log at DEBUG instead, so a kubelet probing twice a second and a Prometheus
scrape every fifteen do not bury the requests a human is actually reading. The
demotion is by exact path match, not prefix: a data-plane route that merely
starts with a probe path's text is not demoted.

The level is the only thing that differs. Probe requests are still logged,
still carry status and latency, and are still recoverable by lowering the
filter — which matters when the question is why a readiness probe started
failing.

Surfaces:
- Rust API: `service_http::trace_layer` - the composed layer that emits the access events.
- Rust API: `service_http::transport::AccessLogOnRequest` / `AccessLogOnResponse` - the request classifier and the response recorder.

Rust internal: the exact-match probe path set and the per-request flag the response recorder reads to choose a level.

EC Dimensions:
- behavior: `cargo test -p service-http --test access_log` - a non-probe request emits an INFO `http.access` event carrying the response status and a latency field, including for a 4xx response.
- security: `cargo test -p service-http --test access_log` - all five probe paths are demoted to DEBUG and are absent from an INFO-filtered subscriber, so a scrape loop cannot flood the operational log.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Data plane logs at INFO | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --test access_log`; `access_log_emits_info_for_non_probe_requests` and `access_log_tracks_4xx_responses` prove the event carries the real status rather than a constant |
| Every probe path is demoted | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --test access_log`; `access_log_probe_paths_all_demoted` iterates all five paths, so demoting four of five would fail |
| Demotion is a level, not a drop | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --test access_log`; `access_log_emits_debug_for_probe_requests` proves the event still exists at DEBUG, so a probe failure remains diagnosable |
| Filter independence | change | 3377 | implemented | verified | smoke | `cargo test -p service-http --test access_log`; `access_log_filter_independent` and `access_log_captured_sample_line` pin the emitted shape against a captured line |

## Not Promised Here

The following behavior exists in the crate but is deliberately given no work
root, because no test in this repository executes it. It is described so the
absence is visible rather than implied:

- **`serve_tls`.** `serve` has a delegation test that drives a real request
  through the shared runtime; the TLS entry point has none. Its ALPN, its
  per-connection configuration source, and its refusal to fall back to
  cleartext are `server-http`'s behavior, exercised there and not here.
- **OTLP span export.** `tests/otlp_tracing.rs` asserts only that the
  logging-only default needs no exporter and that the compatibility surface
  delegates to `service-observability`. Nothing in this repository exports a
  span from this crate to a collector, so the `otlp` feature's export path is
  not a claim made here.
- **The `/docs` page rendering.** The route is asserted to return HTML
  referencing `swagger-ui` and `/openapi.json`. The page loads Swagger UI from
  a public CDN at view time; nothing verifies it renders, and a service in a
  network-isolated cluster should expect it not to.
- **The body cap's coverage of non-extractor body reads.** The layer
  guarantees the byte count is bounded and that a 413 surfacing through axum's
  own `Bytes`/`String`/`Json`/`Form` extractors is rewritten into the
  envelope. A handler that consumes the body through some other path is
  responsible for surfacing that error itself, and no test covers such a
  handler.
- **`tests/behavior_shared_http_service_scaffold_contract.rs`.** It is an
  `#[ignore]`d scaffold, so it is not a gate and is never named as one above.
