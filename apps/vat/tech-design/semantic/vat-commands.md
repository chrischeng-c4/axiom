---
id: semantic-vat-commands
summary: Semantic coverage for "apps/vat/src/commands"
capability_refs:
  - id: "agent-native-gpu-native-dev-containers"
    role: primary
    claim: "host-process-execution-and-gpu-visibility"
    coverage: partial
    rationale: "Semantic takeover coverage for existing source group `apps/vat/src/commands`."
fill_sections: [schema, changes]
---

# Semantic TD: vat/commands

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "vat/commands"
  source_group: "apps/vat/src/commands"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "apps/vat/src/commands/llm.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "service_method"]
        symbols:
          - name: "GUIDE"
            kind: "constant"
            public: false
          - name: "exec"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/vat/src/commands"
      - path: "apps/vat/src/commands/rm.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "exec"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/vat/src/commands"
      - path: "apps/vat/src/commands/ls.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "exec"
            kind: "function"
            public: true
          - name: "status_label"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/vat/src/commands"
      - path: "apps/vat/src/commands/emulator.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "exec"
            kind: "function"
            public: true
          - name: "parse_routes"
            kind: "function"
            public: false
          - name: "exec"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/vat/src/commands"
      - path: "apps/vat/src/commands/run.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "enum_model", "service_method"]
        symbols:
          - name: "Args"
            kind: "struct"
            public: true
          - name: "Target"
            kind: "enum"
            public: true
          - name: "exec"
            kind: "function"
            public: true
          - name: "RunnerArgs"
            kind: "struct"
            public: false
          - name: "DirectArgs"
            kind: "struct"
            public: false
          - name: "exec_direct"
            kind: "function"
            public: false
          - name: "exec_runner"
            kind: "function"
            public: false
          - name: "process_exit_code"
            kind: "function"
            public: false
          - name: "run_configured"
            kind: "function"
            public: false
          - name: "kill_runner_processes"
            kind: "function"
            public: false
          - name: "ordered_required_services"
            kind: "function"
            public: false
          - name: "visit_required_service"
            kind: "function"
            public: false
          - name: "RunnerProc"
            kind: "struct"
            public: false
          - name: "sandbox_wrap"
            kind: "function"
            public: true
          - name: "spawn_runner_process"
            kind: "function"
            public: false
          - name: "wait_runner_processes"
            kind: "function"
            public: false
          - name: "run_setup_step"
            kind: "function"
            public: false
          - name: "ServicePlan"
            kind: "struct"
            public: false
          - name: "ReadyProbe"
            kind: "enum"
            public: false
          - name: "ServiceHandle"
            kind: "struct"
            public: false
          - name: "prepare_service"
            kind: "function"
            public: false
          - name: "prepare_cluster_service"
            kind: "function"
            public: false
          - name: "start_service"
            kind: "function"
            public: false
          - name: "prepare_preset_service"
            kind: "function"
            public: false
          - name: "ResolvedRuntime"
            kind: "enum"
            public: false
          - name: "resolve_preset_runtime"
            kind: "function"
            public: false
          - name: "prepare_preset_docker_service"
            kind: "function"
            public: false
          - name: "prepare_firebase_service"
            kind: "function"
            public: false
          - name: "firebase_emulator_host_var"
            kind: "function"
            public: false
          - name: "builtin_emulator_info"
            kind: "function"
            public: false
          - name: "explicit_network_routes"
            kind: "function"
            public: false
          - name: "preset_auto_routes"
            kind: "function"
            public: false
          - name: "seed_preset_routes_into_proxy"
            kind: "function"
            public: false
          - name: "prepare_builtin_service"
            kind: "function"
            public: false
          - name: "http_mock_env"
            kind: "function"
            public: false
          - name: "prepare_image_service"
            kind: "function"
            public: false
          - name: "docker_run_command"
            kind: "function"
            public: false
          - name: "docker_ready_probe"
            kind: "function"
            public: false
          - name: "container_name"
            kind: "function"
            public: false
          - name: "preset_image"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/vat/src/commands"
      - path: "apps/vat/src/commands/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "cluster"
            kind: "module"
            public: true
          - name: "diff"
            kind: "module"
            public: true
          - name: "emulator"
            kind: "module"
            public: true
          - name: "gpu"
            kind: "module"
            public: true
          - name: "llm"
            kind: "module"
            public: true
          - name: "logs"
            kind: "module"
            public: true
          - name: "ls"
            kind: "module"
            public: true
          - name: "rm"
            kind: "module"
            public: true
          - name: "run"
            kind: "module"
            public: true
          - name: "snapshot"
            kind: "module"
            public: true
          - name: "state"
            kind: "module"
            public: true
          - name: "print_json"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/vat/src/commands"
      - path: "apps/vat/src/commands/state.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "exec"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/vat/src/commands"
      - path: "apps/vat/src/commands/snapshot.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "branch"
            kind: "function"
            public: false
          - name: "snapshot"
            kind: "function"
            public: true
          - name: "fork"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/vat/src/commands"
      - path: "apps/vat/src/commands/gpu.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "exec"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/vat/src/commands"
      - path: "apps/vat/src/commands/diff.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "exec"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/vat/src/commands"
      - path: "apps/vat/src/commands/cluster.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "CREATE_TIMEOUT"
            kind: "constant"
            public: false
          - name: "ClusterRecord"
            kind: "struct"
            public: true
          - name: "create"
            kind: "function"
            public: true
          - name: "ls"
            kind: "function"
            public: true
          - name: "kubeconfig"
            kind: "function"
            public: true
          - name: "delete"
            kind: "function"
            public: true
          - name: "default_cluster_name"
            kind: "function"
            public: false
          - name: "read_registry"
            kind: "function"
            public: false
          - name: "load_record"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/vat/src/commands"
      - path: "apps/vat/src/commands/logs.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "exec"
            kind: "function"
            public: true
          - name: "print_pair"
            kind: "function"
            public: false
          - name: "print_file"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/vat/src/commands"
      - path: "apps/vat/src/commands/build.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["missing-generator:cli:streamed-subprocess-dual-mode"]
        symbols:
          - name: "Args"
            kind: "struct"
            public: true
          - name: "BuildReport"
            kind: "struct"
            public: true
          - name: "ImageBuilder"
            kind: "enum"
            public: false
          - name: "resolve_image_builder"
            kind: "function"
            public: false
          - name: "image_builder_for_runtime"
            kind: "function"
            public: false
          - name: "exec"
            kind: "function"
            public: true
          - name: "build_image"
            kind: "function"
            public: true
          - name: "build_image_with_builder"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "apps/vat/src/commands"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "apps/vat/src/commands/k8s.rs"
    action: create
    section: schema
    description: |
      #1693 hand-written bounded Apple Container K3s command requires an
      independently installed `kubectl` first on PATH and rejects an
      OrbStack-provided binary before K3s use; that is host-tool provenance,
      not a GUI or Docker Engine dependency. It owns explicit image preparation,
      exact machine lifecycle, one-shot and leased private
      kubeconfig/cache injection, verified local-image delivery through private
      OCI staging, backing-id/API-endpoint revalidation, host-command exit
      forwarding, optional `session exec --format json` agent capture, and
      stale/expired-session recovery. Omitted session-exec timeout uses remaining
      lease TTL; explicit timeout is 1..=14400 seconds and cannot exceed it.
      Both text and JSON exec hold exact lease/backing/API/private-credential
      proof and the private lock through owned-process-group cleanup; normal
      exit, deadline, or SIGINT/SIGTERM reaps the group before marker removal.
      A starting/live crash marker blocks later exec/delete/cleanup fail-closed
      rather than claiming recovered-command termination. JSON emits one bounded
      VAT-native result without raw replay or session mutation and masks private
      paths including credential-validation/API-probe failures. Its child receives
      credentials, so it is not credential-free or an untrusted-child boundary.
      The opt-in local-image E2E passed 1/1 (36 filtered) in 49.73s: one
      already-local Apple `alpine:3.20` loaded into one active lease, a pod ran
      it with `imagePullPolicy=Never` and emitted its marker log, then exact
      session cleanup completed. This is not registry-pull generality,
      persistence, GUI, or Docker Engine/API evidence.
      The independent-kubectl leased E2E passed 1/1 (36 filtered) in 29.97s,
      including strict JSON exec with `--timeout 30`, status verification, and
      exact delete.
      `session port-forward run --format json` is separately Service-only and
      loopback-only: it keeps the private lock through shared-PGID cleanup,
      silently rechecks the lease after API proof and immediately before exact
      kubectl/host-child spawn, then emits one post-cleanup
      `vat.k8s.session.port-forward.v1` result with child exit, bounded separate
      streams, no raw replay, masked VAT-owned failures, and status-verify next.
      Its credential-free child output is preserved in successful results. The
      independent-kubectl Service-forward E2E passed 1/1 (36 filtered) in
      49.57s, including one Service-only loopback strict JSON tunnel, confirmed
      cleanup, and closed local ports; it is not a general tunnel claim.
      The command deliberately does not
      extend the Docker-backed cluster subsystem or claim restart-safe
      persistence. On a failed bootstrap, VAT keeps the root error primary,
      then records staged non-sensitive installer evidence through exactly
      guest_install_log, guest_k3s_system, backing_container_logs,
      machine_boot_log, machine_inspect, and container_system_status under a
      six-second total and one-second-per-probe fixed read-only budget before
      exact cleanup. It excludes private kubeconfig/cache and host credentials,
      leaves the existing 300-second bootstrap behavior unchanged, does not
      rerun k3s --version or add a wrapper/recovery command, and does not make
      the session persistent.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/k8s/session_exec.rs"
    action: create
    section: schema
    description: |
      #1693 hand-written leased-session exec owns concurrent bounded child stream
      capture and one `vat.k8s.session.exec.v1` document after exact active-
      lease/backing/API/private-credential proof. Omitted timeout defaults to
      remaining lease TTL; explicit 1..=14400 seconds cannot exceed it. Every
      command owns/reaps its process group and retains the private lock through
      cleanup; a crash marker blocks later lifecycle operations fail-closed rather
      than claiming termination. JSON preserves child exit, masks private paths
      including credential-validation/API-probe failures, does not mutate
      session.json, and does not replay raw output. The independent-kubectl
      leased E2E passed 1/1 (36 filtered) in 29.97s with JSON `--timeout 30`.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/k8s/port_forward_json.rs"
    action: create
    section: schema
    description: |
      #1693 hand-written post-cleanup JSON capture for the Service-only
      loopback tunnel. Text port-forward remains unchanged; only `--format json`
      emits one `vat.k8s.session.port-forward.v1` document after shared-PGID
      cleanup confirms, with child exit, separate 64 KiB serialized streams, no
      raw replay, and status-verify next. VAT masks its setup/API/tunnel/cleanup
      failures while preserving opaque credential-free child output; lease checks
      after API proof and immediately before kubectl/child spawn prevent a
      crossed TTL tunnel. Partial reader setup reaps the direct child and finishes
      outer cleanup before reader join. The independent-kubectl Service-forward
      E2E passed 1/1 (36 filtered) in 49.57s, including one Service-only
      loopback text and strict one-document JSON tunnel with a credential-free
      child, confirmed cleanup, and closed local ports. It remains bounded
      one-guest evidence, not a general tunnel, persistent Kubernetes,
      ingress/LB, public listener, or same-UID OS-sandbox claim.
    impl_mode: hand-written
  - path: "apps/vat/src/docker_shim.rs"
    action: modify
    section: schema
    description: |
      #1685 adds five strict Apple-native JSON observations. Direct container
      inventory is
      only `docker ps --format json` / equals with optional exactly-once
      `--all` or `-a`; only `docker container ls` and `docker container list`
      share it, while `docker container ps --format json` remains rejected;
      inherited text behavior is unchanged. It normalizes inventory to `container
      list --format json [--all]`, validates
      one opaque native JSON value, and byte-for-byte replays stdout with no VAT
      wrapper or Docker Engine ps schema. Templates/table output, filters, quiet
      plus JSON, duplicate/unknown flags, and positionals fail before runtime.
      Inventory is read-only, not ownership/health/readiness/liveness proof.
      A five-second deadline with bounded isolated-process-group cleanup governs
      root exit and both pipe EOFs; each stream is capped at 256 KiB and
      malformed, oversized, or escaped-pipe stdout is suppressed. Image inventory
      is only `docker images --format json` / equals; only `docker image ls` and
      `docker image list` share it, while text/quiet image listing remains
      unchanged. It normalizes to `container image list --format json`,
      bounded-captures and validates one opaque native JSON value, then
      byte-for-byte replays stdout with no VAT wrapper or Docker Engine image
      schema. Template/table/YAML/TOML output, filters, quiet, verbose, all,
      digests, no-trunc, positionals, duplicates, unknown flags, and `--` fail
      before runtime. It makes no ownership/provenance/security/executability/
      registry/build-readiness/health/readiness/liveness claim. The same
      five-second bounded isolated-process-group cleanup covers root exit and
      both pipe EOFs; each stream is capped at 256 KiB and malformed, oversized,
      or escaped-pipe stdout is suppressed. Direct image inspect accepts only
      `docker image inspect --format json IMAGE` / equals: exactly one JSON
      selector must precede exactly one opaque safe image reference (nonempty,
      no leading `-`, whitespace, or control characters). Templates, `--`, extra
      references, and every other option fail before runtime. VAT strips the
      selector, invokes only `container image inspect IMAGE`, bounded-captures
      and validates one opaque Apple-native JSON document, then byte-for-byte
      replays complete native stdout without a VAT wrapper or Docker image-inspect
      schema. The same five-second bounded isolated observer covers root exit and
      both pipe EOFs; each stream is capped at 256 KiB, valid JSON plus a nonzero
      child exit preserves status, and malformed, oversized, or escaped-pipe
      capture suppresses raw stdout. It makes no Docker template/Engine API,
      provenance, security, registry, build-completion, readiness, or
      secret-redaction claim. Direct container inspect accepts only
      `docker inspect --format json CONTAINER` / equals and the same form through
      `docker container inspect`: exactly one safe explicit id follows exactly one
      VAT-only selector before runtime, and unformatted inspect remains inherited.
      It invokes `container inspect CONTAINER`, bounded-captures and validates one
      opaque Apple-native JSON value, then byte-for-byte replays stdout without a
      VAT wrapper or Docker Engine inspect schema. `--type`, `--size`, templates/
      table/YAML/TOML, filters, a second id, `--`, and unknown flags fail before
      runtime. The same five-second bounded isolated observer covers root exit and
      both pipe EOFs; each stream is capped at 256 KiB, valid JSON plus a nonzero
      child exit preserves status, and malformed, oversized, or flood output
      suppresses raw stdout. It makes no ownership/provenance/security/image/
      registry/build-status/health/readiness/liveness/port-reachability claim and
      gives no secret-redaction guarantee. Direct logs JSON is a separate finite
      VAT wrapper, not a sixth Apple-native form: it accepts only `docker logs
      --format json --tail LINES CONTAINER` / equals forms and the same form
      through `docker container logs`, with exactly one format and tail before one
      safe final id and `LINES` in 1..=1000; unformatted logs remains inherited.
      It invokes only `container logs -n LINES CONTAINER`, never forwards the
      selector, and returns exactly one `vat.docker.logs.v1` / `vat_json` wrapper
      with untrusted `apple_container_stdio`, bounded diagnostic stderr,
      truncation/lossy flags, backend/container/requested_tail/runtime/child
      outcome, and a safe inspect next—not Docker schema or multiplex/demux.
      Ordinary child nonzero preserves wrapper plus exit; follow, boot,
      timestamps, since/until, templates, duplicate/misordered selectors,
      unsafe/second ids, and every other modifier fail before runtime; timeout,
      setup, or escaped-pipe capture yields no partial wrapper after five-second
      plus one-second bounded dual-stream suffix and serialized-string caps.
      Direct exec JSON is a separate finite VAT wrapper for `docker exec --format
      json --timeout SECONDS CONTAINER -- COMMAND [ARG...]` / equals forms and the
      same form through `docker container exec`: one format and one timeout occur
      exactly once in either order before a safe id, `SECONDS` is 1..=1200, and the
      Docker-facing delimiter plus a nonempty raw command are mandatory; raw or
      unformatted exec remains inherited. VAT strips selectors and the delimiter,
      then invokes Apple `container exec CONTAINER COMMAND [ARG...]`. One
      `vat.docker.exec.v1` / `vat_json` wrapper contains requested timeout,
      `timeout_scope=host-container-client-observation`, backend/container/runtime
      and child outcome, bounded untrusted stdout/stderr suffixes with
      truncation/lossy flags, no secret-redaction guarantee, and safe inspect next.
      Ordinary child nonzero preserves wrapper plus exit; timeout or setup/capture
      failure emits no partial wrapper. Both pipes drain concurrently and each
      serialized JSON string value is capped at 64 KiB. The timeout only bounds the
      host Apple Container client observation and does not claim guest command
      termination. TTY, interactive, detach, env/user/workdir, templates,
      duplicate/misordered selectors, malformed delimiters, and other exec flags
      fail before runtime.
      Direct run JSON is a separate foreground owner-cleaned one-shot for only
      `docker run --format json --timeout SECONDS IMAGE [COMMAND...]` / equals
      forms. Exactly one format and one 1..=1200 timeout may occur in either
      order before IMAGE; optional command argv follows IMAGE directly. A Docker
      `--` before IMAGE or immediately after IMAGE fails; after the first
      non-`--` command token, later `--` remains opaque child argv. Detach, TTY,
      interactive, caller name/label, ports, network, mounts, env, and every
      other run option fail before runtime. VAT generates a
      high-entropy name and independent owner label, captures bounded
      stdout/stderr, and emits exactly one `vat.docker.run.v1` / `vat_json`
      document only after exact owner-label cleanup confirms absence. Ordinary
      child nonzero preserves wrapper plus exit only then; timeout, setup, or
      cleanup uncertainty emits no partial wrapper. Only Apple's explicit
      `Error: container not found: <name>` diagnostic proves an already-absent
      generated container. Its timeout is host client observation only, with no
      guest-wide termination, crash-recovery cleanup, Docker Engine parity, or
      secret-redaction claim. Focused `docker_run_json` passed 5 plus 1 ignored
      in 1.80s; local `alpine:3.20` real E2E passed 1/1 (56 filtered) in 2.30s
      with one result document and exact cleanup.
      Strict direct build JSON is a separate bounded VAT receipt, not a sixth
      native-JSON observation: only direct `docker build --format json --timeout
      SECONDS --tag TAG [--file DOCKERFILE] [--build-arg K=V ...] [--target STAGE]
      [--platform PLATFORM] [--label K=V ...] CONTEXT` / documented equals forms
      pass. Format json, positive whole timeout 1..=1200, and tag are exactly
      once; file/target/platform are at most once; build args/labels repeat; and
      every option precedes one canonical existing local-directory context.
      `--`, missing/duplicate/misordered selectors, a second context, and
      unsupported flags fail before a builder; raw builds without either selector
      remain inherited. VAT strips only JSON/deadline selectors and maps supported
      fields to public `container build`. After the Apple client exits it emits one
      bounded `vat.docker.build.v1` / `vat_json` receipt with untrusted streams,
      truncation/lossy flags, timeout scope, child outcome, and
      `image_lifecycle=retained_no_auto_cleanup`: no product cleanup/ownership
      claim. Success safely points to strict image inspect; normal child failure
      retains receipt/exit but is `build_failed` with `docker --help`, never stale
      inspect. Timeout/setup/capture failure emits no receipt; deadline is host
      client observation only, not cancellation/rollback/removal. No Docker
      Engine/API, provenance, ownership, readiness, security, redaction,
      cancellation, or rollback claim follows. Current evidence: cargo check
      passed; docker_shim 62/62; focused build 4 plus 1 ignored (63 filtered);
      owner guard 1/1 (67 filtered); host receipt E2E 1/1 (67 filtered) in 2.53s.
      Its high-entropy tag/exact owner label plus pre/post absence and pre-delete
      recheck are test-only best effort; Apple has no conditional build/delete, so
      ambiguity leaks and never changes retained/no-auto-cleanup product behavior.
      Strict direct pull JSON is a separate bounded VAT receipt, not native
      Apple JSON: only direct `docker pull --format json --timeout SECONDS IMAGE`
      / documented equals forms pass. Exactly one json format and positive whole
      1..=1200 timeout may occur in either order before one opaque image reference:
      nonempty, no leading dash, whitespace/control, URL-style `://`, or leading
      Git-style `git@` remote are rejected; ordinary OCI `@digest` remains opaque.
      `--`, a second reference, missing/duplicate/misordered selectors, and
      unsupported flags fail before the client; raw direct pull without either
      selector and every `docker image pull` form remain inherited. VAT strips
      only JSON/deadline selectors and maps to public `container image pull IMAGE`.
      After the Apple client exits it emits one bounded `vat.docker.pull.v1` /
      `vat_json` receipt with untrusted streams, truncation/lossy flags, timeout
      scope, child outcome, and `image_lifecycle=not_owned_no_auto_cleanup`: no
      VAT cleanup, ownership, or registry login/auth/credential lifecycle. Success
      safely points to strict image inspect without image-state/download-completion
      proof; normal child failure preserves receipt/exit with pull_failed/docker-
      help and no stale inspect. Timeout/setup/capture/pipe failure emits no
      receipt; deadline observes host client/pipes only, not transfer cancellation,
      download completion, rollback, or local/backend image state. No Engine/API,
      registry-management, provenance, digest, platform, freshness, image-state,
      ownership, security, redaction, cancellation, download-completion, or rollback
      claim follows. Evidence: cargo check; docker_shim 65/65; focused pull 5 plus
      1 ignored (68 filtered); host receipt E2E 1/1 (73 filtered) in 27.14s. The
      real `alpine:3.20` pull uses shared/cacheable state and neither deletes it nor
      asserts ownership on success or failure.
      Resource stats
      is strict `docker stats --no-stream --format json CONTAINER
      [CONTAINER...]` / equals-form observation. It normalizes to Apple
      `container stats --format json --no-stream`, validates one opaque native
      JSON document, and replays no VAT/Docker Engine schema or wrapper. It is
      read-only, not ownership/health/readiness/liveness proof. A five-second
      deadline with bounded isolated-process-group cleanup governs root exit and
      both pipe EOFs; valid stdout is replayed only after complete bounded capture, and an
      escaped pipe holder fails closed. Each stream is capped at 256 KiB, and
      malformed/oversized stdout is suppressed. `cargo check -p vat
      --no-default-features` passed; shared `docker_shim` library validation
      passed 58/58, focused direct-ps integration passed 4/4, focused
      `docker_images_json` integration passed 4/4, focused
      `docker_image_inspect_json` integration passed 4/4 with 1 ignored, focused `docker_inspect`
      integration passed 5/5, focused `docker_logs_json` integration passed 6/6,
      and focused `docker_exec_json` integration passed 4/4. The full serial
      fake-shim aggregate is intentionally not recorded
      because an independent serial run exposed a nondeterministic pre-existing
      Compose JSON logs timing race. The opt-in direct image-inspect E2E
      `RUST_TEST_THREADS=1 VAT_DOCKER_IMAGE_INSPECT_JSON_E2E_REQUIRED=1 cargo test
      -p vat --test vat_docker_shim apple_container_docker_image_inspect_json_contract
      -- --ignored --nocapture` passed 1/1 (61 filtered) in 1.21s: it proves only
      one direct `container image inspect alpine:3.20` invocation and one valid
      native JSON document, not Docker image-inspect schema/template/Engine API,
      provenance, security, registry, build-completion, readiness, or secret
      redaction. `VAT_DOCKER_SHIM_E2E_REQUIRED=1 cargo test
      -p vat --test vat_docker_shim apple_container_docker_run_published_port_contract
      -- --ignored --nocapture` passed 1/1 (50 filtered) on Apple Container 1.1.0
      with a high-entropy nonce+PID owner-labeled temporary nginx container: `ps`
      and `images` are global read-only inventory smoke observations; `inspect`,
      `stats`, VAT `logs`, and direct exec target that labeled container. Exact-label
      rechecks are conservative best-effort precautions, and the emergency guard
      retains the container on uncertainty. Apple Container has no atomic conditional
      delete, so this is not a race-free or impossible-to-misdelete cleanup guarantee;
      the shared/cacheable nginx image is not cleaned up. The host smoke proves one valid
      native JSON document or VAT wrapper only, including an exec wrapper with both
      stdout and stderr markers; fake/unit tests prove byte-preservation and fail-
      closed details. It does not claim guest-timeout termination, Docker Engine
      parity, immutable image identity, or image cleanup.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/docker_shim.rs"
    action: create
    section: schema
    description: |
      #1685 hand-written vat CLI adapter exposes opt-in Docker shim install
      and status commands without advertising a Docker Engine socket or
      daemon. Full behavior is specified by
      vat-headless-docker-command-shim.md.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/compose.rs"
    action: modify
    section: schema
    description: |
      #1685 P2 keeps Docker-shaped `compose ps` separate from generic VAT
      compose access and adds bounded detached `up -d --wait`: profile,
      generation, and ticket pin durable runner/topology observations while
      claims are released between polls; one ready final result owns topology,
      timeout retains runtime/registry, and terminal/replaced/bare deadline
      outcomes have no unsafe next. Under one registry claim ps derives a
      known-profile, registered-service-order topology snapshot. Endpoints are
      all-or-none canonical loopback strings gated by unique Ready VAT-owned
      container_run/exact-MicroVM/no-cleanup-error evidence; degraded,
      inactive, starting, and stopping state never leak partial endpoints. The
      strict Docker-shaped preflight `docker compose --dry-run -f FILE -p
      PROJECT up -d [--build]` parses only the same strict image/build/
      host-facing profiles and emits one `vat.docker-compose.preflight.v1` VAT
      JSON document with validated=true, runtime_started=false,
      registry_written=false, image_built=false, launch_revalidates=true,
      structured launch_argv, and executable next using the parser's canonical
      source path so a cwd change still revalidates the same file. It invokes no Apple
      Container command, does not build/import/start or write a registry,
      rejects wait and every other global/Compose flag, and its returned real
      launch revalidates the file before stateful work.
      opt-in gated real Apple Container dual-service E2E passed 1/1 (50 filtered)
      on this host in 4.54 seconds for host-facing-independent-v1 up -d --wait,
      both loopback endpoints, one-document JSON ps/logs/exec, text logs, text
      exec including a no-final-newline handoff, and exact down cleanup only—not
      service-name DNS, general Compose, Docker Engine API, or Kubernetes. The
      exact no-format ps retains text plus additive profile/topology JSON;
      `ps --format json` and `ps --format=json` instead emit exactly one
      VAT-owned `schema=vat.docker-compose.ps.v1`, `format=vat_json` document
      with the same claim-held proof and no human table. This is not Docker
      Compose JSON/template/table or application-healthcheck compatibility;
      every other ps format fails closed.
      Docker-shaped text `logs SERVICE` preserves observed log bytes, then
      starts its additive VAT handoff JSON on a new line after them.
      `logs --format json [--tail LINES] SERVICE` (including equals forms and
      final service) emits exactly one capture-only
      `vat.docker-compose.logs.v1` VAT JSON document with separate stdout/stderr
      snapshots, default-200/range-1..=1000 tail_lines, per-stream
      truncated/utf8_lossy, capture_only=true, runtime_invoked=false, and
      compose_record_mutated=false. It holds the existing claim/provenance and
      reads VAT-captured logs only: no Apple Container call, project.json
      mutation, topology, or endpoints. VAT first caps each read and line tail,
      then after lossy UTF-8 plus JSON escaping retains a valid UTF-8 suffix
      whose serialized JSON string value remains within the same 64 KiB
      per-stream cap and marks it truncated; next is VAT-native JSON ps. It
      rejects follow, timestamps, and other flags; it is not Docker Compose
      merged/follow/timestamp/template compatibility. The full serial
      `vat_docker_shim` aggregate is intentionally not recorded because an
      independent serial run exposed a nondeterministic pre-existing Compose JSON
      logs timing race; the focused serialized-cap unit passed 1/1 for
      `0xff`-heavy and NUL/control-heavy streams after actual JSON serialization.
      The recorded opt-in real dual-service E2E includes this JSON logs shape
      for its bounded host-facing profile.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/build.rs"
    action: modify
    section: schema
    description: |
      #1529 source ownership is hand-written: ImageBuilder maps
      Auto/Native/Docker to Docker and MicroVm to Apple Container, preflights
      that exact store, and exposes captured build_image_with_builder() without
      crossing image stores.
      Full source is mirrored by projects-vat-src-commands-build-rs.md.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/capabilities.rs"
    action: modify
    section: schema
    description: |
      Full vat capabilities discovery retains its normal Docker daemon probe.
      Its public `services.docker_services` field is an availability string:
      the full probe maps it to `available` or `unavailable`.
      Its Apple Container builder report is a bounded read-only advisory:
      builder status is shared_unknown ownership with automatic_cleanup=false;
      parseable configured resources remain distinct from optional observed
      stats, and optional system df is global_apple_container host evidence,
      never VAT-attributed disk. Unsupported, malformed, or timed-out probes
      produce nonfatal unknown/probe_errors and never start, stop, delete, or
      prune shared builder/cache state; a live state is reported only if the
      installed CLI supports it.
      A selected-plan caller may instead record
      `services.docker_services=not_probed` plus `docker.daemon_probe` with
      state=skipped and a reason: cli remains a PATH observation and
      daemon=false is not unavailable evidence in that state because no Docker
      command ran. This supports read-only Apple-Container-only doctor preflight
      without changing capabilities' standalone full-probe contract.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/doctor.rs"
    action: modify
    section: schema
    description: |
      Doctor builds the selected RunPlan before capability discovery. An explicit
      MicroVm image/preset-only plan checks the read-only container CLI and one
      container system status probe per doctor invocation, skips Docker with
      truthful daemon_probe skipped provenance and
      `services.docker_services=not_probed`, and ignores unselected Docker
      services. `daemon=false` is not unavailable evidence because no Docker
      command ran. It does not autostart Apple Container or fall back to Docker:
      unsupported no-OCI-route MicroVm presets and MicroVm preset named volumes
      fail closed. The shared builder result is an advisory-only successful
      check, so timeout/unknown/probe errors do not alter runtime success.
      Docker runtime, Auto image, eligible Auto preset fallback, and selected
      cluster plans retain normal Docker probing; a cluster needs Docker.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/llm.rs"
    action: modify
    section: schema
    description: |
      The agent guide mirrors the three-profile Docker shim and the bounded
      detached wait contract, plus the selected-plan doctor distinction from
      full capabilities discovery: the latter includes a non-mutating shared
      builder advisory, while Apple-Container-only plans have one read-only
      container status probe, `services.docker_services=not_probed`, truthful
      skipped Docker-probe provenance (not unavailable), fail-closed
      no-OCI/named-volume behavior, and nonblocking builder evidence. Docker/
      Auto/cluster selections retain Docker probing and map that field to
      `available` or `unavailable` without fallback. It also teaches the
      bounded credentialed K3s `session exec --format json` form separately from
      unchanged text exec: one VAT-native bounded result, no raw replay or
      session mutation, deterministic fake/unit evidence only, and no real-host
      JSON-exec claim.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/rm.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/ls.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/emulator.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/run.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/state.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/snapshot.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/gpu.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/diff.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/cluster.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "apps/vat/src/commands/logs.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
```
