---
id: relay-service-auth-role-map
summary: >
  Adopt the shared libs/service-auth bearer contract on relay's /v1 data
  plane: the blanket auth_middleware (Authorization: Bearer extraction ->
  StaticRoleMapVerifier -> shared {error, message} 401/403 -> RoleMapPrincipal
  injected as an Extension) layered on the data-plane router only, the
  archetype's RELAY_AUTH=off|required + RELAY_TOKEN_REGISTRY_FILE env/flag
  contract with startup fail-fast, and per-handler-group authorization on the
  {subject} path param — publish/publish-batch need a write grant,
  consume/lease/ack/lease-batch/ack-batch/heartbeat/len need read, wildcard *
  grants and the admin >= write >= read hierarchy per role_map::Role::covers.
  Probes (/healthz /readyz /metrics /openapi.json /docs) stay auth-exempt.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: relay-service-auth-role-map-flow
entry: boot
nodes:
  boot:
    kind: start
    label: "serve_main: ServeArgs gains --auth (RELAY_AUTH, off|required, default off) and --token-registry-file (RELAY_TOKEN_REGISTRY_FILE) — serve flags with env fallback, like --bind"
  resolve:
    kind: decision
    label: "auth::AuthConfig::resolve(mode, registry_file, legacy_inline): parse off|disabled|required; when required, load the token registry via service_auth::load_registry — JSON { token: { subject, roles { subject-or-* : read|write|admin } } }"
  failfast:
    kind: terminal
    label: "Startup fail-fast: RELAY_AUTH=required with a missing/unparseable/empty registry file exits nonzero with an error naming RELAY_TOKEN_REGISTRY_FILE — a server that can never authenticate anyone is a misconfiguration, not a per-request 401"
  verifier:
    kind: process
    label: "StaticRoleMapVerifier::new(required, tokens) when required; StaticRoleMapVerifier::open() when off (tokenless dev default). AppState carries Arc<StaticRoleMapVerifier>; AppState::new stays open, AppState::with_auth takes the resolved AuthConfig"
  build:
    kind: process
    label: "router(): the blanket service_auth::auth_middleware is route_layer'd on the /v1 data-plane router ONLY (relay auth is all-or-nothing per deployment — the blanket layer composes directly, no per-handler verifier like keep's optional claim-check). metrics::track stays outermost so auth rejections are still counted"
  req:
    kind: decision
    label: "Request arrives: probe route or /v1 data plane?"
  probes:
    kind: process
    label: "/healthz /readyz /metrics /openapi.json /docs answer tokenless and always-on — the probe router from standard_probe_routes (#1205) never gets the auth layer"
  authn:
    kind: decision
    label: "auth_middleware: bearer_token(headers) -> verifier.authenticate -> RoleMapPrincipal injected as an Extension (Open when off-mode without a token). Missing or unknown token under required -> 401 {error: unauthenticated} (shared service-auth shape)"
  authz:
    kind: decision
    label: "Handler-group authorization on the {subject} path param via auth::authorize(principal, subject, needed): publish/publish-batch need Role::Write; consume/lease/ack/lease-batch/ack-batch/heartbeat/len need Role::Read. Wildcard * grants cover every subject; hierarchy admin >= write >= read per role_map::Role::covers. RoleMapPrincipal::Open passes everything"
  forbidden:
    kind: terminal
    label: "Deny -> RoleMapDenied mapped to AuthError::Forbidden: 403 {error: forbidden, message: subject lacks role on subject} — consistent with the #1205 ApiErr {error, message} envelope family"
  handler:
    kind: process
    label: "Authorized request runs the existing Relay op unchanged (JSON + CBOR encodings, streaming consume — which checks read before its Subscribe handshake). relay llm operations topic documents the knobs + client RELAY_URL/RELAY_TOKEN bearer contract"
  done:
    kind: terminal
    label: "Response returned; RELAY_AUTH=off keeps today's tokenless behavior end to end"
edges:
  - { from: boot, to: resolve }
  - { from: resolve, to: failfast, label: "required + bad/missing registry" }
  - { from: resolve, to: verifier, label: "ok" }
  - { from: verifier, to: build }
  - { from: build, to: req, label: "request accepted" }
  - { from: req, to: probes, label: "probe route" }
  - { from: req, to: authn, label: "/v1/{subject}/* route" }
  - { from: authn, to: authz, label: "principal injected" }
  - { from: authn, to: done, label: "401 unauthenticated" }
  - { from: authz, to: forbidden, label: "grant missing" }
  - { from: authz, to: handler, label: "grant covers needed role" }
  - { from: probes, to: done }
  - { from: handler, to: done }
---
flowchart TD
    boot([serve_main: auth flags + env fallback]) --> resolve{AuthConfig resolve: mode + registry load}
    resolve -->|required + bad registry| failfast([exit nonzero naming RELAY_TOKEN_REGISTRY_FILE])
    resolve -->|ok| verifier[StaticRoleMapVerifier: registry when required, open when off; Arc in AppState]
    verifier --> build[router: auth_middleware route_layer on v1 data plane only]
    build --> req{Probe or data plane?}
    req -->|probe| probes[healthz readyz metrics openapi.json docs — tokenless, always-on]
    req -->|v1| authn{auth_middleware: bearer to principal Extension}
    authn -->|missing or unknown token| done401([401 unauthenticated shared shape])
    authn -->|principal| authz{authorize on subject: publish=write, consume family=read}
    authz -->|denied| forbidden([403 forbidden shared shape])
    authz -->|covered| handler[existing Relay op unchanged]
    probes --> done([response])
    handler --> done
```
