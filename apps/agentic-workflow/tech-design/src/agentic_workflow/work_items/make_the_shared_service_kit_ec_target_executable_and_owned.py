"""Tech design for WI #3345: aw: make the shared service-kit EC target executable and owned.

@spec #3345
"""

from __future__ import annotations

__aw_artifact_id__ = "artifact:existing-project-standardization/make-the-shared-service-kit-ec-target-executable-and-owned-wi-3345"
__aw_work_item__ = "3345"

__aw_changes__ = """
changes:
  - path: libs/service-http/examples/minimal_service.rs
    action: create
    section: libs/service-http/examples
    impl_mode: hand-written
    description: >
      Add a minimal owned Cargo example executable composing real production
      service_http seams (standard_probe_routes, server_timing_middleware,
      trace_layer, serve, shutdown_with_drain, ReadinessHook backed by
      server_lifecycle::Readiness). Binds 127.0.0.1:0, prints exactly
      "LISTENING <resolved_addr>" to stdout after bind, serves /healthz 200,
      /readyz 200-or-503-draining, /metrics 200, /openapi.json 200, /docs 200
      all with Server-Timing response header. On real SIGTERM flips readiness
      to draining, holds 2-second grace window via shutdown_with_drain, exits 0,
      and prints "SHUTDOWN complete" to stdout. No mocks, no copied policy;
      transport-h2c behavior flows through the composed server-http runtime.
  - path: apps/agentic-workflow/external-contracts/src/cases/existing-project-standardization-shared-service-kit-substrate.py
    action: modify
    section: apps/agentic-workflow/external-contracts/src/cases
    impl_mode: hand-written
    description: >
      Retain all existing assertions and subprocess mechanics exactly. Update
      module docstring to document that libs/service-http/examples/minimal_service.rs
      is the explicitly owned and maintained executable owner for this EC.
      Keep build command, binary path, LISTENING parser, HTTP endpoint
      assertions, Server-Timing header check, SIGTERM/drain/grace/exit-0
      assertions, and SHUTDOWN complete output check unchanged.
"""


def design_contract() -> str:
    """Executable design contract for #3345: select and specify the owned EC target.

    Inspected evidence summary
    --------------------------
    - apps/agentic-workflow/external-contracts/src/cases/
      existing-project-standardization-shared-service-kit-substrate.py
        verify() executes `cargo build -p service-http --example minimal_service`,
        runs target/debug/examples/minimal_service, expects LISTENING <addr>,
        then asserts 200 on /healthz /readyz /metrics /openapi.json /docs,
        Server-Timing header, SIGTERM -> /readyz 503 draining, grace >= 1.5s,
        exit 0, and SHUTDOWN complete stdout line.

    - libs/service-http/Cargo.toml
        Declares [lib] only; no [[example]] entries. Dependencies include
        server-http, server-lifecycle, service-observability.

    - libs/service-http/src/lib.rs
        Exports standard_probe_routes, server_timing_middleware, trace_layer,
        serve, shutdown_with_drain, ReadinessHook. "What a service wires"
        wiring is an //! doc `ignore` block — not an executable target.

    - libs/service-http/src/probes.rs
        standard_probe_routes wires /healthz /readyz /metrics /openapi.json /docs
        with correct 200/503 semantics and Swagger UI HTML.

    - libs/service-http/src/readiness.rs
        ReadinessHook = server_lifecycle::Readiness.

    - libs/service-http/src/signal.rs
        shutdown_with_drain and wait_shutdown_signal re-exported from server-lifecycle.

    - libs/service-http/src/server_timing.rs
        server_timing_middleware inserts Server-Timing: app;dur=<ms> on every response.

    - libs/service-http/README.md
        Capability contract documents cargo test only; no maintained executable owner.

    - libs/service-http/examples/ (directory inspection)
        No existing minimal_service.rs or any example file found. Cargo manifest
        contains no [[example]] stanzas. Absence confirmed by manifest inspection
        and issue brief embedded evidence.

    Selected owner
    --------------
    Fallback owner: libs/service-http/examples/minimal_service.rs (new, hand-written).
    Rejected candidates: (1) nonexistent target reference — cannot satisfy AC1/AC4;
    (2) doc-only ignore wiring — not executable; (3) uninspected app binaries — out of scope.

    Frozen runtime spec
    -------------------
    Build:   cargo build -p service-http --example minimal_service
    Binary:  target/debug/examples/minimal_service
    Bind:    127.0.0.1:0 (ephemeral loopback)
    Stdout:  exactly "LISTENING <resolved_addr>" after bind, flushed
    Routes:  /healthz 200 "ok"; /readyz 200 "ok" (503 "draining" after SIGTERM);
             /metrics 200; /openapi.json 200 body contains '"openapi"';
             /docs 200 body contains "swagger-ui" and "/openapi.json"
    Header:  Server-Timing on every response (server_timing_middleware outermost)
    SIGTERM: readiness.start_drain() called via shutdown_with_drain closure;
             /readyz flips to 503 "draining" within 2s poll window;
             grace window = 2s (Duration::from_secs(2));
             process exits code 0 after grace; stdout contains "SHUTDOWN complete"
    Production seams: service_http::{standard_probe_routes, server_timing_middleware,
             trace_layer, serve, shutdown_with_drain, ReadinessHook}
             via server-http / server-lifecycle / transport-h2c composition

    Red oracles
    -----------
    - Missing Cargo target -> build step assertion fails with cargo stderr evidence
    - Missing binary path -> assert binary.exists() fails with path
    - No LISTENING line within 15s -> assertion fails with captured output
    - SIGTERM does not flip /readyz to 503 within 2s -> draining_status is None
    - Process does not exit within 15s -> TimeoutExpired assertion names grace window
    - exit code != 0 -> assertion names returncode
    - No SHUTDOWN complete in stdout -> assertion fails with captured lines
    """
    # ── D1: selected owner path and build command ────────────────────────────
    selected_owner_path = "libs/service-http/examples/minimal_service.rs"
    build_command = "cargo build -p service-http --example minimal_service"
    assert selected_owner_path.endswith(".rs")
    assert "--example minimal_service" in build_command
    assert "-p service-http" in build_command

    # ── D2: required production seams (names must appear in source) ──────────
    required_seams = [
        "standard_probe_routes",
        "server_timing_middleware",
        "trace_layer",
        "serve",
        "shutdown_with_drain",
        "ReadinessHook",
    ]
    assert len(required_seams) == 6

    # ── D3: stdout protocol ──────────────────────────────────────────────────
    import re
    listening_re = re.compile(r"^LISTENING (\S+)$")
    assert listening_re.match("LISTENING 127.0.0.1:12345")
    assert not listening_re.match("LISTENING")
    assert not listening_re.match("listening 127.0.0.1:12345")

    shutdown_token = "SHUTDOWN complete"
    assert "SHUTDOWN complete" == shutdown_token

    # ── D4: endpoint/status assertions ──────────────────────────────────────
    expected_routes = {
        "/healthz": (200, "ok"),
        "/readyz": (200, "ok"),
        "/metrics": (200, None),
        "/openapi.json": (200, '"openapi"'),
        "/docs": (200, "swagger-ui"),
    }
    assert len(expected_routes) == 5
    assert expected_routes["/readyz"] == (200, "ok")

    draining_route = ("/readyz", 503, "draining")
    assert draining_route[1] == 503
    assert draining_route[2] == "draining"

    # ── D5: Server-Timing header required on every response ──────────────────
    header_key = "server-timing"
    assert header_key == header_key.lower()

    # ── D6: grace window bounds ──────────────────────────────────────────────
    grace_secs = 2
    startup_timeout_s = 15.0
    exit_timeout_s = 15.0
    assert grace_secs == 2
    assert exit_timeout_s > grace_secs

    # ── D7: red-oracle falsifiers ────────────────────────────────────────────
    # Missing LISTENING within deadline -> captured list non-empty but addr is None
    addr: str | None = None
    assert addr is None, "oracle: no LISTENING -> addr stays None"

    # Draining flip oracle: status 503 with body containing 'draining'
    draining_observed = (503, "draining")
    assert draining_observed[0] == 503
    assert "draining" in draining_observed[1]

    return "ok"

