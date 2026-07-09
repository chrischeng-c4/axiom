// HANDWRITE-BEGIN gap="missing-generator:logic:c0ur1e07" tracker="pending-tracker" reason="courier's cli_std::llm::Topic list (http-api, operations) + the stamped ToolInfo constructor shared by llm/upgrade/issue."
//! courier's agent-facing CLI identity: the build-stamped [`TOOL`] info and
//! the `courier llm` topic list — the single in-code source of truth shared
//! by the standard `llm` / `upgrade` / `issue` commands (CONTRIBUTING.md CLI
//! convention). Compiled into the `courier` bin via `#[path]` include.

/// This binary's identity + build provenance for the standard CLI ops
/// (`upgrade` / `issue`), per the CONTRIBUTING.md CLI convention.
pub const TOOL: cli_std::ToolInfo = cli_std::ToolInfo {
    project: "courier",
    repo: "chrischeng-c4/axiom",
    target: env!("COURIER_TARGET"),
    version: env!("CARGO_PKG_VERSION"),
    git_sha: env!("COURIER_GIT_SHA"),
    built_at: env!("COURIER_BUILT_AT"),
};

/// courier's agent-facing `llm` topics — the single in-code source of truth.
pub const TOPICS: &[cli_std::llm::Topic] = &[
    cli_std::llm::Topic {
        id: "http-api",
        summary: "the GitHub-issues-proxy surface (search/view/create/comment, probes)",
        body: "# courier — HTTP API surface\n\n\
            One port speaks h2c (HTTP/2 cleartext, prior-knowledge) + HTTP/1.1. courier \
            holds the real GitHub credential server-side and forwards to \
            `api.github.com`; it stores nothing of its own — GitHub stays the source of \
            truth.\n\n\
            - `GET /v1/issues/{owner}/{name}?state=&q=&limit=` — search issues in one \
              repo (forwards to `GET /search/issues`, scoped with `repo:owner/name`).\n\
            - `GET /v1/issues/{owner}/{name}/{number}` — view one issue.\n\
            - `POST /v1/issues/{owner}/{name}` — create an issue (`title`/`body`/`labels` \
              body, forwarded verbatim).\n\
            - `POST /v1/issues/{owner}/{name}/{number}/comments` — reopen (if closed) then \
              comment (`{\"body\": \"...\"}`, forwarded verbatim).\n\
            - `/healthz`, `/readyz`, `/openapi.json` — probe + machine-readable contract.\n\n\
            Every route is scoped to `COURIER_ALLOWED_REPOS` (403 outside the \
            allow-list). The full document: `GET /openapi.json` (served by the binary).\n",
    },
    cli_std::llm::Topic {
        id: "operations",
        summary: "run / configure the proxy — flags, env vars, auth, allow-list",
        body: "# courier — operating the server\n\n\
            Bare `courier` runs the server (env-driven; flags override). Key knobs:\n\n\
            - `--bind` (`COURIER_BIND`, default `0.0.0.0:7400`) — h2c/h1 listen address.\n\
            - `--grace-secs` (`COURIER_GRACE_SECS`, default `10`) — graceful-drain window: \
              on SIGTERM `/readyz` flips to 503 for this many seconds before the \
              listener closes, so k8s stops routing new work first.\n\
            - `--github-token` (`COURIER_GITHUB_TOKEN`, required) — the real GitHub \
              credential courier forwards with; a courier that can never call GitHub \
              fails fast at startup rather than 500ing per request.\n\
            - `--allowed-repos` (`COURIER_ALLOWED_REPOS`, default \
              `chrischeng-c4/axiom`) — comma-separated `owner/name` allow-list; requests \
              outside it get 403 before any GitHub call.\n\
            - `--auth` (`COURIER_AUTH`, `off`|`required`, default `off`) — bearer auth on \
              the /v1 data plane (shared service-auth contract). Probes (`/healthz` \
              `/readyz` `/metrics` `/openapi.json` `/docs`) stay tokenless either way.\n\
            - `--token-registry-file` (`COURIER_TOKEN_REGISTRY_FILE`, production \
              `/var/run/secrets/courier/token-registry.json`) — JSON \
              `{token: {subject, roles: {\"<owner>/<name>|*\": \"read|write|admin\"}}}`; \
              validated at startup when auth is required (missing/bad file = exit). \
              search/view need `read`; create/comment need `write`; `admin >= write >= \
              read`, `*` grants cover every repo.\n\
            - clients: `AXIOM_COURIER_URL` for routing + `AXIOM_COURIER_TOKEN` for \
              credentials, sent as `Authorization: Bearer <token>` — `libs/cli-std`'s \
              `issue` triad calls courier when these are set, and falls back to a \
              direct GitHub call unchanged when they are not.\n\n\
            Deploy: `apps/courier/k8s/{base,overlays}` (kustomize) + \
            `apps/courier/Dockerfile` — a plain Deployment + Service (no CRD/operator — \
            courier holds no state and needs no per-instance custom resource).\n",
    },
];
// HANDWRITE-END
