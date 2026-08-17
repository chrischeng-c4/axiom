<!-- HANDWRITE-BEGIN gap="missing-generator:logic:2721745f" tracker="1868" reason="Define the shared service observability capability and ownership boundary." -->
# service-observability

## Brief

`service-observability` is the protocol-neutral composition owner for service
logging, optional OTLP tracing, stable resource identity, metric providers,
server lifecycle connection metrics, and bounded-soak resource sampling. HTTP
request and route adaptation remains in `service-http`; Prometheus primitives
and encoding remain in `metrics-prometheus`.

It does not own the transport, the collector, the metric names a service
chooses, or the log level policy. It owns the shape.

Lifecycle metrics additionally expose bounded phase, generation, transition
count, and transition-age series without reason/detail labels. The async
lifecycle observer records the initial observation and each generation once,
emits structured transition events, and terminates at terminal phases.

## Structured stdout contract

Long-running services use `LogFormat::Json` as the collector-compatible stdout
surface. Each successful event is one compact JSON object followed by one
newline and conforms to `axiom.service.log.v1`; the checked-in schema lives at
`contracts/axiom.service.log.v1.schema.json`. Stable top-level fields include
service identity, severity, event, message, and optional W3C trace/span/request
correlation. Sensitive propagation values are excluded and remaining
attributes are bounded before serialization.

`LogFormat::Pretty` is explicitly development-only and must not be used as a
collector input. HTTP libraries supply correlation fields on request spans;
this crate owns their exporter-independent stdout serialization. Optional OTLP
export augments that path but never changes the JSONL wire contract.

## Capabilities

Every capability belongs to exactly one of two feature roots:

- **Core Features** define the log envelope every axiom service emits: its
  version and layout, the correlation fields that may appear in it, and the
  containment rule that keeps caller-supplied data from breaking or leaking
  through it.
- **Non-Core Features** keep a service observable when the surrounding
  telemetry stack is not. Non-core does not mean optional.

This section contains stable product promises, claim IDs, and verification
surfaces. Delivery planning lives outside this contract and references these
IDs one way.

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Versioned Log Envelope | - | implemented | verified | smoke | ready | core; every event is one JSON line carrying a schema tag and a fixed set of envelope fields, so a collector can parse a service it has never seen |
| Correlation Field Integrity | - | implemented | verified | smoke | ready | core; a trace, span, or request identifier reaches the envelope only in its exact valid form, and an invalid one is omitted rather than published |
| Attribute Containment | - | implemented | verified | smoke | ready | core; caller-supplied fields cannot overwrite an envelope field, cannot carry a credential-bearing key, and cannot grow a log line without bound |
| Degraded Telemetry Fallback | - | implemented | verified | smoke | ready | non-core; a missing, malformed, or uncompiled trace exporter downgrades to structured logs instead of failing startup |
| Connection Lifecycle Metrics | - | implemented | verified | smoke | ready | non-core; accepted, rejected, and closed connections are counted behind the lifecycle seam and rendered as canonical Prometheus text, so every runtime that links this crate exposes the same three series |
| Portable Process Sampling | - | implemented | verified | smoke | ready | non-core; resident memory and CPU time are read through one portable surface whose macOS and Linux output shapes both parse exactly |
| Physical Filesystem Usage | - | implemented | verified | smoke | ready | non-core; total, used, and available bytes are read through safe statvfs without external binaries or unsafe FFI |

### Core Features

#### Versioned Log Envelope

ID: versioned-log-envelope
Root WI: -
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
Every event a service emits in JSON mode is exactly one line of JSON carrying
the schema tag `axiom.service.log.v1`. The envelope always has a timestamp, a
severity, a service identity with a name and a version, an event name, and a
message; the correlation fields are present only when they are valid, and
`attributes` holds everything else. The event name is the caller's `event` field
when it supplied a non-empty one and the tracing metadata name otherwise, and
the message falls back to the event name when no `message` field was recorded,
so neither is ever empty or missing. A service identity with a blank name or a
blank version is rejected at construction, so no line can claim to come from an
unnamed service. Only the JSON format is collector-compatible; the pretty format
is a developer convenience and says so.
Surfaces:
- Rust API: `service_observability::ServiceLogEventV1` / `ServiceLogIdentityV1` - the envelope, serialized and deserialized.
- Rust API: `service_observability::SERVICE_LOG_SCHEMA_V1` - the schema tag every line carries.
- Rust API: `service_observability::service_log_schema_v1` - the checked-in JSON Schema, parsed.
- Rust API: `service_observability::collector_compatible` - whether a chosen log format produces collector-readable output.
- Rust API: `service_observability::ServiceIdentity::new` - reject a blank service name or version.
- Rust API: `service_observability::ServiceJsonFormatter` - the tracing-subscriber event formatter that produces the envelope.
- Contract file: `contracts/axiom.service.log.v1.schema.json` - the published schema.
Rust internal: the fallback chain that fills the event name and the message, and the omission of absent optional fields from the serialized line.
EC Dimensions:
- behavior: `cargo test -p service-observability --test service_log_jsonl` - a formatted event is one line of valid JSON whose schema tag, severity, service identity, event name, and message all hold the expected values, and an event with no explicit `event` or `message` field still produces non-empty ones.
- security: `cargo test -p service-observability --test service_log_jsonl` - an absent correlation field is omitted from the line rather than serialized as null, the serialized envelope keys match the published schema exactly with no additional properties, and the pretty format is not reported as collector-compatible; identity refusal of a blank or whitespace-only name or version is covered by `cargo test -p service-observability --lib`.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Schema-tagged envelope | change | - | implemented | verified | smoke | `cargo test -p service-observability --test service_log_jsonl`; every line carries the `axiom.service.log.v1` tag and the fixed envelope fields, so a collector keys off the tag rather than guessing the producer's layout |
| Non-empty event and message | change | - | implemented | verified | smoke | `cargo test -p service-observability --test service_log_jsonl`; the event name falls back to the tracing metadata name and the message falls back to the event name, so neither field is ever empty even when the caller supplied nothing |
| Rejected blank identity | change | - | implemented | verified | smoke | `cargo test -p service-observability --lib`; a name or version that is empty or only whitespace fails identity construction, so a service cannot emit anonymous lines |
| Format compatibility declaration | change | - | implemented | verified | smoke | `cargo test -p service-observability --test service_log_jsonl`; only the JSON format reports as collector-compatible, so a deployment that leaves the pretty format on can be detected rather than silently producing unparseable output |

#### Correlation Field Integrity

ID: correlation-field-integrity
Root WI: -
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
A correlation identifier enters the envelope only in its exact valid form. A
trace id must be 32 lower-case hex characters, a span id and a parent span id 16,
and none of the three may be all zeros — the W3C "invalid id" value. Trace flags
must be exactly two lower-case hex characters, where all zeros is a legitimate
value. A request id must be non-empty, at most 128 bytes, and free of control
characters. Anything that fails its check is omitted from the envelope, never
truncated into shape and never published as-is. Event-level fields take
precedence over span-level fields, and a request id is looked for under
`request_id`, then `request.id`, then `http.request.id`, in that order, with the
first valid value winning.
Surfaces:
- Rust API: `service_observability::ServiceLogEventV1::trace_id` / `span_id` / `parent_span_id` / `trace_flags` / `request_id` - the correlation fields, each optional.
- Rust API: `service_observability::ServiceJsonFormatter` - performs the validation while formatting.
Rust internal: the lower-hex validator with its exact-length and reject-all-zero rules, the event-before-span precedence, and the three-key request-id search order.
EC Dimensions:
- behavior: `cargo test -p service-observability --test service_log_jsonl` - a valid trace id, span id, parent span id, and trace flags reach the envelope unchanged; an event-level value wins over a span-level one; and each of the three request-id keys is honored in its documented order.
- security: `cargo test -p service-observability --test service_log_jsonl` - an id of the wrong length, containing upper-case hex, containing a non-hex character, or consisting entirely of zeros is omitted; an all-zero `trace_flags` is kept, because zero flags are valid; and a request id that is empty, over-long, or contains a control character is omitted rather than sanitized.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Exact-form identifier validation | change | - | implemented | verified | smoke | `cargo test -p service-observability --test service_log_jsonl`; length, lower-case hex, and the all-zero rejection are all checked, so a malformed id is dropped rather than propagated into a trace backend that would treat it as real |
| Zero-flag distinction | change | - | implemented | verified | smoke | `cargo test -p service-observability --test service_log_jsonl`; the all-zero rejection applies to the three ids and not to `trace_flags`, where `00` means "not sampled" and must survive |
| Event-over-span precedence | change | - | implemented | verified | smoke | `cargo test -p service-observability --test service_log_jsonl`; a field recorded on the event outranks the same field inherited from an enclosing span, so the most specific correlation available is the one published |
| Request-id key order | change | - | implemented | verified | smoke | `cargo test -p service-observability --test service_log_jsonl`; `request_id`, `request.id`, and `http.request.id` are searched in that order and the first valid value wins, so a service that names the field either way correlates identically |
| Control-character rejection | change | - | implemented | verified | smoke | `cargo test -p service-observability --test service_log_jsonl`; a request id carrying a newline or other control character is omitted, so a caller-supplied value cannot inject structure into a downstream log pipeline |

#### Attribute Containment

ID: attribute-containment
Root WI: -
Status: verified
Type: Feature
Feature Class: core
Required Verification: smoke
Promise:
Everything the caller records that is not an envelope field lands under
`attributes`, and it lands there bounded and screened. A key that names an
envelope field — including the three request-id spellings — is dropped rather
than shadowing it, so no caller can rewrite the severity or the service identity
of its own line. A key that names a credential-bearing header is dropped:
`authorization`, `proxy_authorization`, `cookie`, `set_cookie`, `baggage`, and
`tracestate`, compared case-insensitively with `-` normalized to `_`, and
matched not just exactly but as a trailing segment after `.`, `/`, or `_`, so
`http.request.header.authorization` is caught as readily as `Authorization`. At
most 64 attributes survive; a key is capped at 128 bytes and a string value at
4096. Truncation is UTF-8 safe: it steps back to a character boundary rather
than cutting a multi-byte character in half and producing invalid JSON. A
non-scalar value is rendered to a string and bounded like any other, so no
single field can carry an unbounded structure into the log pipeline.
Surfaces:
- Rust API: `service_observability::ServiceLogEventV1::attributes` - the bounded, screened map.
- Rust API: `service_observability::MAX_ATTRIBUTES` / `MAX_ATTRIBUTE_KEY_BYTES` / `MAX_ATTRIBUTE_VALUE_BYTES` / `MAX_EVENT_BYTES` / `MAX_REQUEST_ID_BYTES` - the published bounds.
Rust internal: the reserved-key set, the sensitive-key normalization and suffix matching, the attribute-count cutoff, and the character-boundary walk in the truncator.
EC Dimensions:
- behavior: `cargo test -p service-observability --test service_log_jsonl` - ordinary caller fields reach `attributes` with their scalar types intact, the tracing target is recorded when the caller did not supply one, and a non-scalar value is rendered to a bounded string.
- security: `cargo test -p service-observability --test service_log_jsonl` - a reserved key never appears under `attributes` and never overwrites its envelope field; every sensitive key is dropped in each of its spellings, including a prefixed one; more than 64 attributes are cut to 64; an over-long key or value is truncated to its exact byte bound; and truncating a multi-byte character yields valid UTF-8 rather than a split character.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Reserved-key shadowing refused | change | - | implemented | verified | smoke | `cargo test -p service-observability --test service_log_jsonl`; a caller field named after an envelope field is dropped instead of merged, so severity, service identity, and correlation cannot be forged from inside a log call |
| Sensitive-key exclusion | change | - | implemented | verified | smoke | `cargo test -p service-observability --test service_log_jsonl`; the six credential-bearing names are matched case-insensitively, with `-` normalized to `_`, and as a trailing segment, so a header captured under a namespaced key is excluded as reliably as a bare one |
| Bounded attribute count | change | - | implemented | verified | smoke | `cargo test -p service-observability --test service_log_jsonl`; at most 64 attributes survive, so a caller in a loop cannot make one line grow without limit |
| UTF-8 safe truncation | change | - | implemented | verified | smoke | `cargo test -p service-observability --test service_log_jsonl`; the truncator steps back to a character boundary, so a key or value cut at its byte bound is still valid UTF-8 and the line is still parseable JSON |

### Non-Core Features

#### Connection Lifecycle Metrics

ID: connection-lifecycle-metrics
Root WI: -
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
A runtime built on the shared lifecycle seam counts every connection it accepts,
every connection admission rejects, and every connection that finishes, and
publishes them as three canonical Prometheus counters:
`service_connections_accepted_total`, `service_connections_rejected_total`, and
`service_connections_closed_total`. The counter names, `HELP` text, `TYPE`
lines, and their order are fixed by this crate rather than chosen per service,
so one dashboard reads every axiom service. Rendering is a pure function of the
current counter values and goes through the shared Prometheus encoder, never
through hand-built strings. The provider seam is protocol-neutral: a runtime
that has no metrics still satisfies it and renders an empty body instead of
failing.
Surfaces:
- Rust API: `service_observability::MetricsProvider` - the protocol-neutral rendering seam, defaulting to an empty body.
- Rust API: `service_observability::LifecycleMetrics` - the canonical accepted/rejected/closed counters and their current values.
- Rust API: `impl server_lifecycle::ConnectionMetrics for LifecycleMetrics` - the bridge that turns lifecycle events into counter increments.
Rust internal: the fixed sample order and the delegation to `metrics_prometheus::render` rather than a locally formatted exposition.
EC Dimensions:
- behavior: `cargo test -p service-observability --lib` - each lifecycle event increments only its own counter, the readers return the exact counts, and the rendered body is the canonical exposition with all three series in their fixed order.
- security: `cargo test -p service-observability --lib` - a provider with no metrics renders an empty body rather than panicking, and the exposition is produced by the shared encoder, so a counter value cannot inject `HELP`, `TYPE`, or series lines into the rendered text.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Canonical counter set | change | - | implemented | verified | smoke | `cargo test -p service-observability --lib`; the three connection counters carry names and help text owned by this crate, so a fleet-wide alert does not have to know which service produced the series |
| One event, one counter | change | - | implemented | verified | smoke | `cargo test -p service-observability --lib`; accepted, rejected, and closed increment independently, so a rejected connection is never also counted as accepted and admission pressure stays distinguishable from ordinary churn |
| Encoder-owned exposition | change | - | implemented | verified | smoke | `cargo test -p service-observability --lib`; the body comes from the shared Prometheus encoder in a fixed sample order, so the text is scrapeable and no counter value can forge exposition structure |
#### Degraded Telemetry Fallback

ID: degraded-telemetry-fallback
Root WI: -
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
A service starts and logs whether or not trace export is available. With no
configured OTLP endpoint the mode is logging-only. With an endpoint that is not
an absolute `http` or `https` URI with an authority, the mode records that the
endpoint is invalid and keeps logging. With a valid endpoint but no compiled
exporter, the mode records that the feature is disabled and keeps logging. In
every unavailable case the subscriber is still installed and the reason is
emitted as a warning, so the operator learns why traces are missing from the
same stdout stream they already collect. Configuration is transport-neutral: a
level, a format, and an optional endpoint, with no knowledge of how the service
serves traffic.
Surfaces:
- Rust API: `service_observability::tracing_mode` - resolve configuration to a mode without installing anything.
- Rust API: `service_observability::TracingMode` / `OtelFallback` - the resolved mode and the reason for a fallback.
- Rust API: `service_observability::init_tracing` / `init_tracing_with_identity` - install the subscriber for the resolved mode.
- Rust API: `service_observability::ObservabilityConfig` / `LogFormat` - the transport-neutral settings.
Rust internal: the endpoint validity check, and the branch that installs the plain subscriber before warning on every unavailable path.
EC Dimensions:
- behavior: `cargo test -p service-observability --lib` - no endpoint resolves to logging-only; a valid endpoint resolves to the exporter mode when it is compiled in and to the disabled-feature fallback when it is not; and the resolved configuration round-trips its level, format, and endpoint unchanged.
- security: `cargo test -p service-observability --lib` - a malformed endpoint, a scheme other than http or https, and an authority-less URI each resolve to the invalid-endpoint fallback rather than being dialed, and no unavailable path turns into a startup failure.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Endpoint validity gate | change | - | implemented | verified | smoke | `cargo test -p service-observability --lib`; an endpoint must parse as a URI with an http or https scheme and an authority before any exporter is built, so a typo becomes a logged fallback instead of a dial attempt |
| Fallback never fails startup | change | - | implemented | verified | smoke | `cargo test -p service-observability --lib`; every unavailable path installs the logging subscriber first and then warns with the reason, so a missing collector degrades observability without taking the service down |
| Transport-neutral configuration | change | - | implemented | verified | smoke | `cargo test -p service-observability --lib`; the config carries a level, a format, and an optional endpoint and nothing about how the service is served, so the same settings apply to any runtime that links this crate |
| Compiled exporter path | change | - | implemented | verified | smoke | `cargo test -p service-observability --features otlp`; with the exporter compiled in, a valid endpoint resolves to the exporter mode rather than the disabled-feature fallback, so the default-feature suite cannot green the branch it never builds |

#### Portable Process Sampling

ID: portable-process-sampling
Root WI: -
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
Resident memory and accumulated CPU time for a process are read through one
portable surface, with no unsafe FFI and no platform-specific branch in the
caller. Resident size arrives in KiB and is published in bytes. CPU time is
parsed from every shape the surface emits — `MM:SS.ss`, `HH:MM:SS`, and
`D-HH:MM:SS` — and converted to seconds. A sample that cannot be parsed, or a
process that does not exist, is an error the caller sees, not a zero it might
mistake for an idle process.
Surfaces:
- Rust API: `service_observability::ProcessUsage` - the sample, as CPU seconds and resident bytes.
- Rust API: `service_observability::process_usage` - sample one process by pid.
Rust internal: the CPU-time parser's day-prefix split and its two- and three-field clock shapes, and the saturating KiB-to-bytes conversion.
EC Dimensions:
- behavior: `cargo test -p service-observability --lib` - each of the three CPU-time shapes converts to the exact expected number of seconds, and a KiB resident size converts to the exact byte count.
- security: `cargo test -p service-observability --lib` - a missing field, a non-numeric field, and an unrecognized clock shape each produce an error rather than a silently wrong sample, and the KiB conversion saturates instead of wrapping.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Exact CPU-time parsing | change | - | implemented | verified | smoke | `cargo test -p service-observability --lib`; the two-field, three-field, and day-prefixed shapes all convert to exact seconds, so a long-running soak reports real CPU rather than a shape the parser happened to accept |
| Errors instead of zero samples | change | - | implemented | verified | smoke | `cargo test -p service-observability --lib`; a missing, non-numeric, or unrecognized field is an error, so a failed sample cannot be read as an idle process and quietly pass a resource budget |
| Saturating unit conversion | change | - | implemented | verified | smoke | `cargo test -p service-observability --lib`; the KiB-to-byte multiplication saturates, so an implausible reading clamps at the maximum instead of wrapping to a small number |
| Soak runner parses | change | - | implemented | verified | smoke | `bash -n libs/service-observability/scripts/soak-metrics.sh`; the shared soak runner that consumes these samples is syntax-checked, so a broken runner fails here rather than part-way through a long soak |

#### Physical Filesystem Usage

ID: physical-filesystem-usage
Root WI: -
Status: verified
Type: Feature
Feature Class: non-core
Required Verification: smoke
Promise:
Total, used, and available bytes for the filesystem carrying a given path are
read through one safe surface, with no unsafe FFI, no external binary
invocation, and no platform-specific branch in the caller. Total space is
derived from total blocks, available headroom from unprivileged unreserved
blocks, and used space from the difference between total and free blocks. A
path that does not exist or cannot be queried is an error the caller sees, not
a zero it might mistake for an empty or full volume.
Surfaces:
- Rust API: `service_observability::FilesystemUsage` - the sample, as total, used, and available bytes.
- Rust API: `service_observability::filesystem_usage` - sample filesystem usage for a path.
Rust internal: the conversion of kernel statvfs block counts and fragment size into byte counts with saturating arithmetic.
EC Dimensions:
- behavior: `cargo test -p service-observability --test filesystem_usage_is_physical` - total bytes matches independent filesystem accounting, and writing bytes to the sampled filesystem reduces available bytes by the written size.
- security: `cargo test -p service-observability --test filesystem_usage_is_physical` - an unreadable or non-existent path produces an error rather than a zeroed sample.

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Physical write tracking | change | - | implemented | verified | smoke | `cargo test -p service-observability --test filesystem_usage_is_physical`; writing bytes to the mounted filesystem reduces available bytes, proving the reading reflects physical storage |
| Independent accounting agreement | change | - | implemented | verified | smoke | `cargo test -p service-observability --test filesystem_usage_is_physical`; total bytes matches independent df block accounting on the same path |
| Errors instead of zeroed samples | change | - | implemented | verified | smoke | `cargo test -p service-observability --test filesystem_usage_is_physical`; a missing or unreadable path is an error, so a capacity controller cannot mistake a failure for a full volume |

<!-- HANDWRITE-END -->
