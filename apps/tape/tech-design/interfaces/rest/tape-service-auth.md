---
id: tape-service-auth
summary: >
  Add TAPE_AUTH=off|required + token-registry bearer auth on tape's
  /topics data plane, following the shared libs/service-auth role-map
  contract already adopted by relay/lumen/keep: the blanket
  auth_middleware (Authorization: Bearer extraction ->
  StaticRoleMapVerifier -> shared {error, message} 401/403 ->
  RoleMapPrincipal injected as an Extension) layered on the data-plane
  router only, with TAPE_TOKEN_REGISTRY_FILE startup fail-fast, and
  per-handler authorization on the {topic} path param -- append needs a
  write grant (producer side), replay/checkpoint-get/checkpoint-put all
  need a read grant (consumer side, mirroring relay's ack/heartbeat/
  lease-batch precedent for consumer-local cursor mutations). Probes
  (/healthz /readyz /metrics /openapi.json /docs) stay auth-exempt.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-service-auth-flow
entry: boot
nodes:
  boot:
    kind: start
    label: "serve_main: ServeArgs gains --auth (TAPE_AUTH, off|required, default off) and --token-registry-file (TAPE_TOKEN_REGISTRY_FILE) -- serve flags with env fallback, like --bind/--store/--grace-secs"
  resolve:
    kind: decision
    label: "tape::auth::AuthConfig::resolve(mode, registry_file, legacy_inline): parse off|disabled|required; when required, load the token registry via service_auth::load_registry -- JSON { token: { subject, roles { topic-or-* : read|write|admin } } }, naming TAPE_TOKEN_REGISTRY_FILE and the TAPE_TOKENS legacy/dev inline fallback"
  failfast:
    kind: terminal
    label: "Startup fail-fast: TAPE_AUTH=required with a missing/unparseable/empty registry file exits nonzero naming TAPE_TOKEN_REGISTRY_FILE -- a server that can never authenticate anyone is a misconfiguration, not a per-request 401"
  verifier:
    kind: process
    label: "StaticRoleMapVerifier::new(required, tokens) when required; StaticRoleMapVerifier::open() when off (tokenless dev default). AppState carries Arc<StaticRoleMapVerifier>; AppState::new stays open (tests/existing callers unchanged), AppState::with_auth(journal, store, auth) takes the resolved AuthConfig"
  build:
    kind: process
    label: "router(): service_auth::auth_middleware is route_layer'd on the /topics data-plane router ONLY (BEFORE metrics::track so rejected requests are still counted); the probe surface (standard_probe_routes, merged separately) never gets the layer and stays tokenless"
  req:
    kind: decision
    label: "Request arrives: probe route or /topics data plane?"
  probes:
    kind: process
    label: "/healthz /readyz /metrics /openapi.json /docs answer tokenless and always-on regardless of TAPE_AUTH"
  authn:
    kind: decision
    label: "auth_middleware: bearer_token(headers) -> verifier.authenticate -> RoleMapPrincipal injected as an Extension (Open when off-mode without a token). Missing or unknown token under required -> 401 {error: unauthenticated} (shared service-auth shape)"
  authz:
    kind: decision
    label: "Per-handler authorization on the {topic} path param (the resource) via tape::auth::authorize(principal, topic, needed): append needs Role::Write (producer side); replay, checkpoint_get, and checkpoint_put all need Role::Read -- a consumer advancing its own replay checkpoint mutates only consumer-local cursor state, the same shape as relay's ack/heartbeat/lease-batch/ack-batch, which sit in relay's READ group rather than a third tier. Wildcard * grants cover every topic; hierarchy admin >= write >= read per role_map::Role::covers"
  forbidden:
    kind: terminal
    label: "Deny -> RoleMapDenied mapped to AuthError::Forbidden: 403 {error: forbidden, message: topic lacks role on topic} -- consistent with the service_http::ApiErr {error, message} envelope family"
  handler:
    kind: process
    label: "Authorized request runs the existing append/replay/checkpoint_get/checkpoint_put handler unchanged (TapeJournal API, persistence to --store) -- no new domain behavior, per the #1325 shell boundary"
  done:
    kind: terminal
    label: "Response returned; TAPE_AUTH=off keeps today's tokenless behavior end to end (regression)"
edges:
  - { from: boot, to: resolve }
  - { from: resolve, to: failfast, label: "required + bad/missing registry" }
  - { from: resolve, to: verifier, label: "ok" }
  - { from: verifier, to: build }
  - { from: build, to: req, label: "request accepted" }
  - { from: req, to: probes, label: "probe route" }
  - { from: req, to: authn, label: "/topics/{topic}/* route" }
  - { from: authn, to: authz, label: "principal injected" }
  - { from: authn, to: done, label: "401 unauthenticated" }
  - { from: authz, to: forbidden, label: "grant missing" }
  - { from: authz, to: handler, label: "grant covers needed role" }
  - { from: probes, to: done }
  - { from: handler, to: done }
---
flowchart TD
    boot([serve_main: auth flags + env fallback]) --> resolve{AuthConfig resolve: mode + registry load}
    resolve -->|required + bad registry| failfast([exit nonzero naming TAPE_TOKEN_REGISTRY_FILE])
    resolve -->|ok| verifier[StaticRoleMapVerifier: registry when required, open when off; Arc in AppState]
    verifier --> build[router: auth_middleware route_layer on /topics data plane only]
    build --> req{Probe or data plane?}
    req -->|probe| probes[healthz readyz metrics openapi.json docs -- tokenless, always-on]
    req -->|topics| authn{auth_middleware: bearer to principal Extension}
    authn -->|missing or unknown token| done401([401 unauthenticated shared shape])
    authn -->|principal| authz{authorize on topic: append=write, replay/checkpoint-get/checkpoint-put=read}
    authz -->|denied| forbidden([403 forbidden shared shape])
    authz -->|covered| handler[existing TapeJournal-backed handler unchanged]
    probes --> done([response])
    handler --> done
```
