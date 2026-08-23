# service-http status

## Scope

This document describes the current source contract for the reusable
`service-http` crate. It does not claim that the named gate ran in this working
session.

Use the [README](README.md) for the composition workflow. Use the
[roadmap](ROADMAP.md) for future outcomes and explicit non-goals.

## State definitions

| State | Meaning |
|---|---|
| Supported | The current source has a public contract, an implementation, and a named executable gate for the stated scope. |
| Limited | The current source supports the stated scope, but the Limits cell names a material boundary. |
| Not supported | The behavior is not part of the current product contract. The Evidence cell points to a future outcome or a non-goal. |

## Support matrix

| Surface | ID | State | Supported scope | Limits | Evidence |
|---|---|---|---|---|---|
| Standard operational routes | `standard-operational-routes` | Supported | Callers can mount health, readiness, metrics, OpenAPI, and docs routes from supplied inputs. | Domain routes and authentication policy remain app-owned. | `cargo test -p service-http` |
| Structured error envelope | `structured-error-envelope` | Supported | `ApiErr` renders a caller-selected status through the shared `{error,message}` JSON body. | The adopting service classifies domain errors and chooses safe messages. | `cargo test -p service-http` |
| Request body limit | `request-body-limit` | Supported | The streaming body layer rejects declared-length or mid-read overruns with a structured `413`. | The adopting service selects the byte limit and exemption boundary. | `cargo test -p service-http --test body_limit` |
| Admission response | `admission-response` | Supported | Caller-defined token buckets can deny a classified request with a structured `429` and `Retry-After`. | The caller owns classification, policy values, and the opaque admission key. | `cargo test -p service-http --lib` |
| Inbound request trace context | `inbound-request-trace-context` | Supported | The trace layer accepts a valid W3C version-00 `traceparent` or creates local trace and span IDs for request correlation. | Invalid input is treated as absent. This scope does not inject trace context into outbound requests. | `cargo test -p service-http --test request_trace_context` |
| Server timing | `server-timing` | Supported | Opt-in middleware adds total request duration and can expose caller-recorded phases after explicit full disclosure. | Phase disclosure is app policy because this crate does not know the request auth result. | `cargo test -p service-http --test server_timing` |
| Lifecycle adapters | `lifecycle-adapters` | Supported | HTTP serving, probes, signals, drain, and terminal reporting can share one caller-owned lifecycle. | `server-http` and `server-lifecycle` own listener and lifecycle mechanics. | `cargo test -p service-http --test lifecycle_probes --test lifecycle_signal` |
| OpenAPI middleware response projection | `openapi-middleware-response-projection` | Not supported | The crate's shared `401`, `413`, `429`, and `500` responses are not projected automatically into every mounted OpenAPI operation. | Apps must declare reachable middleware responses themselves today. | [OpenAPI middleware response projection](ROADMAP.md#openapi-middleware-response-projection) |
| Outbound trace propagation | `outbound-trace-propagation` | Not supported | The crate has no outbound HTTP client adapter that injects the current `traceparent`. | Current tracing covers the inbound request span only. | [Outbound trace propagation](ROADMAP.md#outbound-trace-propagation) |

## Evidence policy

The commands above are required gates for each supported scope. This document
does not store execution output. CI and local test logs own run evidence.

Update a row with any public API, behavior, or evidence change. Move a future
outcome into current support only after the implementation and executable gate
exist.
