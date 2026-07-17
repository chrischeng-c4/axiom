---
id: "1685"
summary: Opt-in, headless, fail-closed Docker command shim over Apple Container.
fill_sections: [scenarios, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: headless-docker-command-shim
    coverage: partial
    rationale: "Agents need familiar shell commands for the common Apple Container lifecycle, without presenting VAT as a Docker Engine, GUI, or desktop product."
---

# VAT Headless Docker Command Shim

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: opt_in_multicall_install
    given:
      - "a user selects an explicit directory on PATH"
      - "the directory either lacks docker or already contains VAT's own symlink"
    when:
      - "the user runs vat docker install-shim --dir <dir>"
    then:
      - "VAT creates or confirms docker -> vat"
      - "VAT never overwrites a regular file, real Docker client, or foreign symlink"
      - "stdout names an executable PATH handoff"
  - id: common_agent_shell_lifecycle
    given:
      - "docker is VAT's installed multicall symlink"
      - "Apple Container is installed and running"
    when:
      - "an agent uses an allowlisted command such as build, run, ps, logs, inspect, exec, or rm"
    then:
      - "VAT translates it to one public container CLI invocation"
      - "child stdio and normal numeric exit code are preserved"
      - "docker run -p requires an explicit nonzero host port"
  - id: strict_non_streaming_stats_observation
    given:
      - "docker is VAT's installed multicall symlink"
      - "an agent names one or more explicit Apple Container ids"
    when:
      - "the agent runs docker stats --no-stream --format json CONTAINER [CONTAINER...] or the exact --format=json spelling"
    then:
      - "only those two pre-id flags and explicit ids are accepted; streaming, templates, --all, duplicate/unknown flags, and options after ids fail before Apple Container starts"
      - "VAT runs canonical container stats --format json --no-stream ids, validates one opaque Apple-native JSON document, and replays it without a VAT/Docker Engine wrapper or schema"
      - "the observation is read-only and is not an ownership, health, readiness, or liveness proof"
      - "one five-second deadline plus bounded isolated-process-group cleanup governs direct-child exit and both pipe EOFs; an escaped pipe holder fails closed without stdout replay"
      - "stdout and stderr each retain at most 256 KiB; VAT replays only complete validated native JSON, so malformed or oversized stdout is suppressed rather than partially replayed"
  - id: strict_direct_ps_json_inventory
    given:
      - "docker is VAT's installed multicall symlink"
      - "an agent requests a direct Apple Container inventory snapshot"
    when:
      - "the agent runs docker ps --format json or --format=json with optional exactly-once --all or -a, or the same JSON form through docker container ls or docker container list"
    then:
      - "VAT rejects templates/table output, filters, quiet plus JSON, duplicate/unknown flags, and positionals before Apple Container starts; docker container ps --format json remains rejected"
      - "VAT runs canonical container list --format json [--all], validates one opaque Apple-native JSON value, and byte-for-byte replays stdout without a VAT wrapper or Docker Engine ps schema"
      - "the snapshot is read-only and is not an ownership, health, readiness, or liveness proof"
      - "one five-second deadline plus bounded isolated-process-group cleanup governs direct-child exit and both pipe EOFs; malformed, oversized, or escaped-pipe stdout fails closed without replay"
      - "stdout and stderr each retain at most 256 KiB; inherited plain-text behavior is unchanged"
  - id: strict_direct_images_json_inventory
    given:
      - "docker is VAT's installed multicall symlink"
      - "an agent requests an Apple Container image inventory snapshot"
    when:
      - "the agent runs docker images --format json or --format=json, or the same JSON form through docker image ls or docker image list"
    then:
      - "VAT rejects template/table/YAML/TOML output, filters, quiet, verbose, all, digests, no-trunc, positionals, duplicates, unknown flags, and -- before Apple Container starts"
      - "VAT runs canonical container image list --format json, bounded-captures and validates one opaque Apple-native JSON value, then byte-for-byte replays stdout without a VAT wrapper or Docker Engine image schema"
      - "the snapshot makes no ownership, provenance, security, executability, registry, build-readiness, health, readiness, or liveness claim"
      - "one five-second deadline plus bounded isolated-process-group cleanup governs direct-child exit and both pipe EOFs; malformed, oversized, or escaped-pipe stdout fails closed without replay"
      - "stdout and stderr each retain at most 256 KiB; inherited text/quiet image-list behavior is unchanged"
  - id: strict_direct_image_inspect_json
    given:
      - "docker is VAT's installed multicall symlink"
      - "an agent requests one direct Apple Container image document"
    when:
      - "the agent runs docker image inspect --format json IMAGE or --format=json"
    then:
      - "VAT accepts exactly one JSON selector before exactly one opaque safe image reference: it is nonempty and has no leading -, whitespace, or control characters"
      - "VAT rejects templates, --, a second reference, and every other option before Apple Container starts"
      - "VAT strips its selector, invokes only container image inspect IMAGE, bounded-captures and validates one opaque Apple-native JSON document, then byte-for-byte replays complete native stdout"
      - "one five-second bounded isolated observer governs root exit and both pipe EOFs; stdout and stderr each retain at most 256 KiB; valid JSON with a nonzero child exit preserves that status, while malformed, oversized, or escaped-pipe capture suppresses raw stdout"
      - "the result makes no Docker image-inspect schema/template/Engine API, provenance, security, registry, build-completion, readiness, or secret-redaction claim"
  - id: strict_direct_container_inspect_json
    given:
      - "docker is VAT's installed multicall symlink"
      - "an agent requests one direct Apple Container container document"
    when:
      - "the agent runs docker inspect --format json CONTAINER or --format=json, or the same JSON form through docker container inspect"
    then:
      - "VAT accepts exactly one safe explicit container id after exactly one JSON selector; that selector must precede the id, is VAT-only, and is never forwarded, while unformatted inspect keeps its existing generic behavior"
      - "VAT rejects --type, --size, templates/table/YAML/TOML, filters, a second id, --, and unknown flags before Apple Container starts"
      - "VAT runs canonical container inspect CONTAINER, bounded-captures and validates one opaque Apple-native JSON value, then byte-for-byte replays stdout without a VAT wrapper or Docker Engine inspect schema"
      - "one five-second bounded isolated observer governs root exit and both pipe EOFs; stdout and stderr each retain at most 256 KiB; valid JSON with a nonzero child exit preserves that status, while malformed, oversized, or flood output suppresses raw stdout"
      - "the result makes no ownership, provenance, security, image, registry, build-status, health, readiness, liveness, or port-reachability claim and gives no secret-redaction guarantee"
  - id: strict_direct_logs_json_snapshot
    given:
      - "docker is VAT's installed multicall symlink"
      - "an agent needs one finite direct Apple Container log suffix"
    when:
      - "the agent runs docker logs --format json --tail LINES CONTAINER or mixed equals/separated spellings, or the same form through docker container logs"
    then:
      - "VAT accepts format and tail exactly once before one safe final id, requires LINES in 1..=1000, keeps unformatted logs on the inherited text path, and never forwards the JSON selector"
      - "VAT invokes only canonical container logs -n LINES CONTAINER and returns exactly one VAT vat.docker.logs.v1/vat_json wrapper, not a sixth Apple-native JSON form or a Docker multiplex/demux contract"
      - "the wrapper carries untrusted apple_container_stdio, bounded diagnostic stderr, truncation/lossy flags, backend/container/requested_tail/runtime/child outcome, and a safe inspect next; ordinary child nonzero retains wrapper plus exit code"
      - "a five-second observer plus one-second cleanup drains both pipes, retains suffixes, and caps each capture plus actual serialized JSON-string value at 64 KiB; timeout, setup, or escaped-pipe failure emits no partial wrapper"
      - "follow, boot, timestamps, since/until, templates, and every other modifier fail before Apple Container; the snapshot makes no Docker schema/multiplex/demux, ownership/provenance/security/image/registry/build, health/readiness/liveness/port-reachability, or secret-redaction claim"
  - id: strict_direct_exec_json_snapshot
    given:
      - "docker is VAT's installed multicall symlink"
      - "an agent needs one bounded noninteractive foreground command snapshot"
    when:
      - "the agent runs docker exec --format json --timeout SECONDS CONTAINER -- COMMAND [ARG...] or equals forms, or the same form through docker container exec"
    then:
      - "VAT requires exactly one format and one timeout in either order before one safe id, requires SECONDS in 1..=1200 and a nonempty command after the Docker-facing literal --, and leaves unformatted/raw exec on the inherited generic path"
      - "VAT removes its selectors and Docker-only delimiter, invokes only container exec CONTAINER COMMAND [ARG...], and emits exactly one vat.docker.exec.v1/vat_json wrapper with host-container-client-observation timeout scope, bounded untrusted stdout/stderr suffixes, child outcome, and a safe inspect next"
      - "ordinary child nonzero preserves wrapper plus exit; timeout or setup/capture failure emits no partial wrapper; both streams drain concurrently and each serialized JSON string is capped at 64 KiB"
      - "the timeout bounds only VAT's host Apple Container client observation and makes no guest-command termination claim; TTY, interactive, detach, env/user/workdir, templates, duplicate/misordered selectors, malformed delimiters, and all other exec flags fail before runtime"
      - "the command makes no Docker Engine stream/TTY, ownership, readiness, health, or secret-redaction claim"
  - id: strict_direct_run_json_one_shot
    given:
      - "docker is VAT's installed multicall symlink"
      - "an agent needs one foreground image command with a machine-readable result"
    when:
      - "the agent runs direct `docker run --format json --timeout SECONDS IMAGE [COMMAND...]` or equals forms"
    then:
      - "VAT accepts exactly one format and one 1..=1200 timeout in either order before IMAGE; optional command argv follows IMAGE directly, and only direct docker run owns this JSON surface"
      - "the parser rejects a Docker `--` before IMAGE or immediately after IMAGE; after the first non-`--` command token, later `--` remains opaque child argv. It also rejects detach, TTY, interactive, caller name/label, ports, network, mounts, env, and every other run option before Apple Container starts"
      - "VAT creates a high-entropy name plus independent owner label, captures bounded stdout/stderr, and emits one vat.docker.run.v1/vat_json document only after exact owner-label cleanup confirms absence"
      - "ordinary child nonzero preserves wrapper plus exit only after confirmed cleanup; timeout, setup, or cleanup uncertainty emits no partial wrapper; only Apple's explicit Error: container not found: <name> diagnostic proves an already-absent generated container"
      - "the timeout is host Apple Container client observation only; the one-shot makes no guest-wide termination, crash-recovery cleanup, Docker Engine parity, or secret-redaction claim"
  - id: strict_direct_build_json_receipt
    given:
      - "docker is VAT's installed multicall symlink"
      - "an agent needs one machine-readable direct Apple Container build receipt"
    when:
      - "the agent runs direct `docker build --format json --timeout SECONDS --tag TAG [--file DOCKERFILE] [--build-arg K=V ...] [--target STAGE] [--platform PLATFORM] [--label K=V ...] CONTEXT` or documented equals forms"
    then:
      - "VAT requires exactly one json format, positive whole 1..=1200 timeout, and tag; permits file/target/platform once and repeated build args/labels; and requires every option before exactly one canonical existing local-directory context"
      - "VAT rejects --, missing/duplicate/misordered selectors, a second context, and unsupported flags before a builder starts; raw builds with neither receipt selector retain inherited translation"
      - "VAT strips only JSON/deadline selectors, invokes public `container build --tag TAG [--file ...] [--build-arg ...] [--target ...] [--platform ...] [--label ...] CONTEXT`, then emits one bounded vat.docker.build.v1/vat_json receipt after the client exits"
      - "the receipt marks image_lifecycle=retained_no_auto_cleanup and has bounded untrusted streams with truncation/lossy flags, timeout scope, and child outcome; success has strict image-inspect next, while ordinary child failure retains receipt/exit but is terminal build_failed with docker --help rather than stale image inspect"
      - "timeout, setup, or capture failure emits no receipt. The deadline is host Apple Container client observation only: it makes no cancellation, rollback/removal, Docker Engine/API, provenance, ownership, readiness, security, or secret-redaction claim"
      - "host-test cleanup is not product behavior: a high-entropy tag and exact io.cclab.vat.e2e-owner label need exact native absence before build, owner-label recheck before delete, and exact absence after; Apple has no conditional build/delete, so best-effort races leak on ambiguity"
  - id: strict_direct_pull_json_receipt
    given:
      - "docker is VAT's installed multicall symlink"
      - "an agent needs one machine-readable direct Apple Container pull receipt"
    when:
      - "the agent runs direct `docker pull --format json --timeout SECONDS IMAGE` or documented equals forms"
    then:
      - "VAT requires exactly one json format and positive whole 1..=1200 timeout, in either order before one opaque image reference; it rejects empty, leading-dash, whitespace/control, URL-style `://`, and leading Git-style `git@` remote forms while ordinary OCI `@digest` remains opaque"
      - "VAT rejects --, a second image reference, missing/duplicate/misordered selectors, and unsupported flags before the client; raw direct pulls without either receipt selector and every docker image pull form retain inherited behavior"
      - "VAT strips only JSON/deadline selectors, invokes public `container image pull IMAGE`, then emits one bounded vat.docker.pull.v1/vat_json receipt after the client exits"
      - "the receipt has bounded untrusted streams, truncation/lossy flags, timeout scope, and child outcome with image_lifecycle=not_owned_no_auto_cleanup: the image is shared, no VAT cleanup or ownership exists, and VAT implements no registry login/auth/credential lifecycle"
      - "success has strict image-inspect next without image-state/download-completion proof; ordinary child failure retains receipt/exit but is terminal pull_failed with docker --help rather than stale inspect; timeout, setup, capture, or pipe failure emits no receipt"
      - "the deadline observes only the host client and copied pipes: it makes no transfer cancellation, download-completion, rollback, local/backend image-state, Docker Engine/API, registry-management, provenance, digest, platform, freshness, ownership, security, or secret-redaction claim"
  - id: unsupported_engine_semantics_fail_closed
    given:
      - "an agent calls docker info, docker version, a general Compose form, an unsupported flag, Docker template/filter output, an Engine-only network mode, or a global prune"
    when:
      - "VAT parses the argv"
    then:
      - "VAT exits nonzero before invoking container"
      - "the error states that no Docker Engine socket/API or parity is claimed"
  - id: strict_single_image_v1_compose_lifecycle
    given:
      - "docker is VAT's installed multicall symlink"
      - "a literal Compose file has exactly one image service, one nonzero host:container port, optional literal environment, and no profile marker"
    when:
      - "an agent runs docker compose -f FILE -p PROJECT up -d, then ps, logs SERVICE, exec -T SERVICE -- COMMAND, and down"
    then:
      - "VAT preflights the file before creating a registry or invoking Apple Container"
      - "VAT records strict-single-image-v1 shim provenance"
      - "VAT routes through its typed MicroVM compose lifecycle rather than a synthetic one-step argv"
      - "up emits a runnable next; ps/logs are terminal observed; exec proves an exact ready VAT-owned MicroVM service, forwards the child exit code, and emits a runnable ps next; down is terminal cleaned_up"
      - "build, multi-service topology, depends_on, volumes, extensions, interpolation, and --env-file fail closed"
  - id: strict_single_build_v1_compose_source_build_lifecycle
    given:
      - "docker is VAT's installed multicall symlink"
      - "a literal Compose file has exactly one short build context, no image field, one nonzero host:container port, and optional literal environment"
    when:
      - "an agent runs docker compose -f FILE -p PROJECT up -d --build, then ps, exec -T SERVICE -- COMMAND, and down"
    then:
      - "VAT captures and preflights the build-only profile once before materialization or runtime launch, so a later source-path replacement cannot widen the validated Compose shape"
      - "VAT records strict-single-build-v1 shim provenance"
      - "VAT builds into the selected Apple Container image store through its typed MicroVM Compose lifecycle, then runs that exact image"
      - "the successful up result returns the build image's project-scoped exact images tag and cleanup_next=down plus docker image rm for that exact tag; VAT never uses Docker or a global prune"
      - "image plus build, build mappings/args/custom Dockerfiles, multi-service topology, volumes, interpolation, and non-detached/recreate flags fail closed"
  - id: host_facing_independent_v1_compose_lifecycle
    given:
      - "docker is VAT's installed multicall symlink"
      - "a Compose file has exactly the top-level marker x-vat-compose-profile: host-facing-independent-v1"
      - "the file has two through four literal-image services, each with one nonzero host:container port and no duplicate host port"
    when:
      - "an agent runs docker compose -f FILE -p PROJECT up -d, then ps, logs SERVICE, exec -T SERVICE -- COMMAND, and down"
    then:
      - "VAT preflights the exact marker and every service before creating a registry or invoking Apple Container"
      - "each service is independently reachable only through its published loopback host port; VAT never creates a bridge network or service-name DNS"
      - "the successful vat_docker_compose JSON is an explicit negative contract with profile=host-facing-independent-v1, service_name_dns=false, and host_loopback_only=true"
      - "dependencies, networks, volumes, build, interpolation, env-file, extra extensions, and every other topology form fail closed before runtime launch"
      - "the deterministic fake lifecycle is supplemented by an opt-in gated real Apple Container dual-service E2E that passed 1/1 (50 filtered) on this host in 4.54 seconds"
      - "that real evidence proves host-facing-independent-v1 up -d --wait, both loopback endpoints, one-document JSON ps/logs/exec, text logs, text exec including a no-final-newline handoff, and down cleanup of exact containers, ports, and registry only; it does not establish service-name DNS, general Compose, Docker Engine API, or Kubernetes"
  - id: bounded_compose_wait
    given:
      - "a Docker-shaped Compose file has passed its selected strict profile validation, any source build has completed, and an agent requests docker compose -f FILE -p PROJECT up -d --wait [--wait-timeout SECONDS]"
    when:
      - "VAT launches the detached runner and polls its durable lifecycle/topology evidence"
    then:
      - "explicit -d/--detach remains mandatory; --wait occurs at most once; --wait-timeout is legal only with wait as positive whole seconds, defaulting to 300 and capped at 1200"
      - "the deadline starts after validated import/build and immediately before launch, covers detached handoff and observations, and proves only durable VAT runner readiness/topology—not Docker healthchecks, application HTTP, service DNS, or generic Compose readiness"
      - "the wait target is pinned to exact profile, launch generation, and ticket; registry claims are released between polls, so a prior waiter cannot attach after down, generic re-import, or relaunch"
      - "verified readiness emits exactly one final up JSON result with wait { requested, timeout_seconds, outcome=ready } and ready topology; source-build cleanup_next appears only in this verified-ready case"
      - "timeout retains the launched runtime and registry. A ps handoff is emitted only after a current pinned-target observation; terminal, lifecycle-replaced, and bare-deadline outcomes carry no unsafe next. Degraded has no endpoint"
  - id: compose_ps_topology_contract
    given:
      - "a Docker-shaped Compose project has a known captured shim profile and is queried with exactly docker compose -p PROJECT ps"
    when:
      - "VAT gathers the observation while holding the compose registry claim"
    then:
      - "the final additive vat_docker_compose JSON retains the known profile and adds topology { phase, ready, services }; phase is inactive, starting, ready, degraded, or stopping"
      - "services are ordered by registered Compose service IDs, not by persisted runtime-evidence order; each carries name/state and an endpoint is a canonical 127.0.0.1:<port> string"
      - "all endpoints are emitted only when every expected service has exactly one Ready VAT-owned container_run record for its exact MicroVM, a nonzero loopback port, and no cleanup error"
      - "a proof failure changes a nominal ready lifecycle to degraded with ready=false and no endpoints; inactive, starting, and stopping also have no endpoints"
      - "the topology is lifecycle and ownership evidence, not an application health check; Docker ps --format and generic, missing, or unknown shim provenance fail closed before topology output"
  - id: compose_exec_json_agent_capture
    given:
      - "a Docker-shaped Compose project has known shim provenance and exactly one unique Ready VAT-owned MicroVM record for the requested service"
    when:
      - "an agent runs docker compose -p PROJECT exec -T --format json SERVICE -- COMMAND or the exact --format=json spelling"
    then:
      - "text exec preserves observed child bytes, then starts its additive VAT handoff JSON on a new line after them; JSON accepts no other placement of the format marker, requires it immediately after -T, requires SERVICE next and the literal Docker-facing -- delimiter, and fails closed for default TTY, other exec flags, misordering, or a missing delimiter"
      - "one same-read registry snapshot under the existing claim supplies known profile provenance and exact unique ready-MicroVM proof through Apple Container child spawn; VAT parses and validates the Docker-facing delimiter but does not forward it, invoking Apple Container as container exec CONTAINER COMMAND [ARG...], and releases the claim immediately after spawn before waiting"
      - "VAT drains stdout and stderr concurrently, caps each serialized JSON string value at 64 KiB, and emits exactly one VAT-native vat.docker-compose.exec.v1 document with profile, child_exit_code, separate stdout/stderr, per-stream truncated/utf8_lossy, runtime_invoked=true, and compose_record_mutated=false"
      - "JSON replays no raw child output and exposes no topology or endpoints; it is not Docker Compose exec output compatibility"
      - "The full serial shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race; the precise serialized-cap unit passed 1/1, while the bounded real-host JSON-exec coverage is recorded by the dual-service E2E"
  - id: shim_provenance_boundary
    given:
      - "a Compose registry record was created by a known Docker shim profile, or carries an unknown future profile"
    when:
      - "an agent attempts generic vat compose or Docker-shaped lifecycle commands"
    then:
      - "generic vat compose up, ps, logs, and down cannot operate a known shim record; only the matching Docker-shaped lifecycle may do so"
      - "an explicit inactive generic vat compose import transfers a known shim record back to generic lifecycle by clearing known shim provenance"
      - "an inactive unknown-profile vat compose down performs registry-only cleanup: it removes project.json, preserves vat.toml, and never touches a runtime"
      - "an unknown active profile fails closed and requires a matching or newer VAT that recognizes the profile, or restoration of the matching Docker shim"
  - id: permanently_headless
    given:
      - "the shim is installed"
    then:
      - "no GUI, Desktop, dashboard, tray, menu-bar, daemon, or Engine socket is introduced"
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat docker install-shim --dir <dir>
    behavior:
      - "Create an opt-in docker -> vat symlink only at an absent target or an existing symlink resolving to the current VAT executable."
      - "Emit JSON with a safe next PATH command."
  - name: vat docker status --dir <dir>
    behavior:
      - "Read-only ownership check for the selected docker symlink."
  - name: docker <allowlisted command>
    behavior:
      - "Route by argv[0] basename before clap parses VAT's normal CLI."
      - "Translate only build, pull/push, run/create, lifecycle, logs, exec, copy, inspect, registry, and basic explicit-name image/container/network/volume commands."
      - "Reject global prune and commands with no VAT-owned resource boundary."
      - "Do not implement Docker Engine info/version/context, general Compose, buildx, SDK/Testcontainers/devcontainer behavior, output schemas, or a socket API."
  - name: docker ps --format json [--all|-a]
    behavior:
      - "Accept only --format json or --format=json plus optional exactly-once --all or -a; docker container ls and docker container list share this JSON form, while docker container ps --format json is rejected."
      - "Reject templates/table output, filters, quiet plus JSON, duplicate/unknown flags, and positionals before runtime invocation."
      - "Normalize to container list --format json [--all], validate one opaque Apple-native JSON value, and byte-for-byte replay complete native stdout without a VAT wrapper or Docker Engine ps schema."
      - "Bound root exit and both pipe EOFs to five seconds with isolated-process-group cleanup; cap each stream at 256 KiB and suppress malformed, oversized, or escaped-pipe stdout. This is read-only inventory, not ownership/health/readiness/liveness proof."
  - name: "docker images --format json | --format=json"
    behavior:
      - "Accept only the exact JSON format form; docker image ls and docker image list share it, while existing text/quiet image-list behavior is unchanged."
      - "Reject template/table/YAML/TOML output, filters, quiet, verbose, all, digests, no-trunc, positionals, duplicates, unknown flags, and -- before runtime invocation."
      - "Normalize to container image list --format json, bounded-capture and validate one opaque Apple-native JSON value, then byte-for-byte replay complete native stdout without a VAT wrapper or Docker Engine image schema."
      - "Bound root exit and both pipe EOFs to five seconds with isolated-process-group cleanup; cap each stream at 256 KiB and suppress malformed, oversized, or escaped-pipe stdout. This is read-only image inventory, with no ownership/provenance/security/executability/registry/build-readiness/health/readiness/liveness proof."
  - name: "docker image inspect --format json IMAGE | --format=json"
    behavior:
      - "Accept only direct docker image inspect with exactly one JSON selector before exactly one opaque safe IMAGE: it is nonempty and has no leading -, whitespace, or control characters."
      - "Reject templates, --, a second image reference, and every other option before runtime invocation."
      - "Strip the selector and invoke only container image inspect IMAGE, bounded-capture and validate one opaque Apple-native JSON document, then byte-for-byte replay complete native stdout without a VAT wrapper or Docker image-inspect schema."
      - "Bound root exit and both pipe EOFs to five seconds with isolated-process-group cleanup; cap each stream at 256 KiB, preserve valid native JSON plus a nonzero child exit, and suppress malformed, oversized, or escaped-pipe capture. This makes no Docker template/Engine API, provenance, security, registry, build-completion, readiness, or secret-redaction claim."
  - name: "docker inspect --format json CONTAINER | --format=json"
    behavior:
      - "Accept only the exact JSON format form through docker inspect or docker container inspect: exactly one safe explicit container id follows exactly one selector, which must precede the id and is VAT-only rather than backend argv; unformatted inspect remains inherited behavior."
      - "Reject --type, --size, templates/table/YAML/TOML, filters, a second id, --, and unknown flags before runtime invocation."
      - "Normalize to container inspect CONTAINER, bounded-capture and validate one opaque Apple-native JSON value, then byte-for-byte replay complete native stdout without a VAT wrapper or Docker Engine inspect schema."
      - "Bound root exit and both pipe EOFs to five seconds with isolated-process-group cleanup; cap each stream at 256 KiB. Preserve valid native JSON plus a nonzero child exit status; suppress raw stdout for malformed, oversized, or flood output. This makes no ownership/provenance/security/image/registry/build-status/health/readiness/liveness/port-reachability claim and gives no secret-redaction guarantee."
  - name: "docker logs --format json --tail LINES CONTAINER | equals forms"
    behavior:
      - "Accept only direct logs or docker container logs with one --format json and one --tail LINES in either separated or equals spelling, before one safe final id; LINES is 1..=1000 and unformatted logs remains inherited text behavior."
      - "Reject follow, boot, timestamps, since/until, templates, duplicate/misordered selectors, unsafe/second ids, and every other modifier before runtime invocation."
      - "Normalize only to container logs -n LINES CONTAINER; never forward the Docker JSON selector. Apple stdout is text only, so emit exactly one VAT vat.docker.logs.v1/vat_json wrapper rather than Apple-native JSON or a Docker multiplex/demux schema."
      - "Drain both pipes for five seconds plus one-second isolated cleanup, retain suffixes, and cap each capture plus actual serialized JSON-string value at 64 KiB. The wrapper has untrusted apple_container_stdio, bounded diagnostic stderr, truncation/lossy flags, backend/container/requested_tail/runtime/child outcome, and safe inspect next; child nonzero preserves wrapper+exit, while timeout/setup/escaped-pipe failure emits no partial wrapper and makes no ownership/provenance/security/image/registry/build/health/readiness/liveness/port-reachability/secret-redaction claim."
  - name: docker exec --format json --timeout SECONDS CONTAINER -- COMMAND [ARG...]
    behavior:
      - "Accept only docker exec or docker container exec with exactly one --format json/--format=json and one --timeout SECONDS/--timeout=SECONDS in either order before one safe container id; require SECONDS in 1..=1200, a literal Docker-facing -- immediately after the id, and at least one raw command argument; unformatted/raw exec remains inherited."
      - "Reject TTY, interactive, detach, environment/user/workdir, templates, duplicate/misordered selectors, unsafe ids, missing delimiter/command, and every other exec flag before runtime invocation."
      - "Strip the selectors and Docker-only delimiter, then invoke only container exec CONTAINER COMMAND [ARG...]. Emit exactly one VAT vat.docker.exec.v1/vat_json wrapper with requested timeout, timeout_scope=host-container-client-observation, backend/container/runtime/child outcome, bounded untrusted stdout/stderr suffixes with truncation/lossy flags, no secret-redaction guarantee, and safe inspect next."
      - "Drain both pipes concurrently and cap each serialized JSON string value at 64 KiB. Ordinary child nonzero preserves wrapper+exit; timeout or setup/capture failure emits no partial wrapper. The timeout bounds only the host Apple Container client observation and makes no guest-command termination, Docker Engine stream/TTY, ownership, readiness, health, or redaction claim."
  - name: docker run --format json --timeout SECONDS IMAGE [COMMAND...]
    behavior:
      - "Accept only direct docker run with exactly one --format json/--format=json and one --timeout SECONDS/--timeout=SECONDS in either order before IMAGE; require SECONDS in 1..=1200 and pass optional command argv directly after IMAGE."
      - "Reject a Docker `--` before IMAGE or immediately after IMAGE; after the first non-`--` command token, later `--` remains opaque child argv. Also reject detach, TTY, interactive, caller name/label, ports, network, mounts, env, and every other run option before runtime invocation."
      - "Generate a high-entropy name and independent owner label, run foreground, capture bounded stdout/stderr, and emit exactly one vat.docker.run.v1/vat_json document only after exact owner-label cleanup confirms absence."
      - "Preserve ordinary child nonzero only after confirmed cleanup; timeout/setup/cleanup uncertainty emits no partial wrapper. Accept only Apple's explicit Error: container not found: <name> diagnostic as absence. The timeout is host-client observation only and makes no guest-wide termination, crash-recovery, Docker Engine, or redaction claim."
  - name: docker build --format json --timeout SECONDS --tag TAG [--file DOCKERFILE] [--build-arg K=V ...] [--target STAGE] [--platform PLATFORM] [--label K=V ...] CONTEXT
    behavior:
      - "Accept only direct selector-bearing build receipts: exactly one --format json/--format=json, one --timeout SECONDS/--timeout=SECONDS in 1..=1200, and one --tag TAG; allow --file, --target, and --platform once plus repeated --build-arg and --label; require every option before one existing local-directory CONTEXT. Raw builds without either selector retain inherited translation."
      - "Reject --, missing/duplicate/misordered selectors, a second context, and unsupported flags before runtime. Strip only JSON/deadline selectors and invoke public container build --tag TAG [--file ...] [--build-arg ...] [--target ...] [--platform ...] [--label ...] CONTEXT."
      - "After the Apple client exits, emit one bounded vat.docker.build.v1/vat_json receipt with untrusted stdout/stderr, truncation/lossy flags, timeout scope, and child outcome. image_lifecycle=retained_no_auto_cleanup: product builds get no auto-cleanup or ownership claim."
      - "Success returns strict docker image inspect next. Ordinary child failure retains receipt plus exit but returns terminal=build_failed and next=docker --help, never a stale inspect handoff. Timeout/setup/capture failure emits no receipt; deadline is observation only, not builder cancellation or rollback/removal. It makes no Docker Engine/API, provenance, readiness, security, redaction, cancellation, or rollback claim."
  - name: docker pull --format json --timeout SECONDS IMAGE
    behavior:
      - "Accept only direct docker pull with exactly one --format json/--format=json and one --timeout SECONDS/--timeout=SECONDS in 1..=1200, in either order before one opaque IMAGE; reject empty, leading-dash, whitespace/control, URL-style `://`, and leading Git-style `git@` remote forms while ordinary OCI `@digest` remains opaque. Raw direct pull without either selector and every docker image pull form remain inherited."
      - "Reject --, a second image reference, missing/duplicate/misordered selectors, and every unsupported flag before Apple Container. Strip only JSON/deadline selectors and invoke public container image pull IMAGE."
      - "After the Apple client exits, emit one bounded vat.docker.pull.v1/vat_json receipt with untrusted stdout/stderr, truncation/lossy flags, timeout scope, and child outcome. image_lifecycle=not_owned_no_auto_cleanup: the shared image has no VAT ownership or cleanup; VAT provides no registry login/auth/credential lifecycle."
      - "Success returns strict docker image inspect next without proving image state or download completion. Ordinary child failure retains receipt plus exit but returns terminal=pull_failed and next=docker --help, never stale inspect. Timeout/setup/capture/pipe failure emits no receipt; deadline observes only host client/pipes, not transfer cancellation, completion, rollback, or local/backend image state. It makes no Docker Engine/API, registry management, provenance, digest, platform, freshness, ownership, security, or redaction claim."
  - name: docker stats --no-stream --format json CONTAINER [CONTAINER...]
    behavior:
      - "Accept only --no-stream plus --format json or --format=json before one or more explicit ids; reject streaming, templates, --all, duplicate/unknown flags, and options after ids before runtime invocation."
      - "Normalize to container stats --format json --no-stream ids, observe it for at most five seconds with bounded isolated-process-group cleanup for root exit and both pipe EOFs, and fail closed if an escaped pipe holder prevents complete capture."
      - "Validate one complete Apple-native JSON document before replaying exact native stdout with no VAT/Docker Engine wrapper/schema. Capture each stream at 256 KiB; suppress malformed or oversized stdout. This is read-only observation, not ownership/health/readiness/liveness proof."
  - name: docker compose -f FILE -p PROJECT up -d [--wait [--wait-timeout SECONDS]]
    behavior:
      - "Accept a project already matching [a-z0-9][a-z0-9_-]* and exactly one of two no-build profiles: strict-single-image-v1 (one literal-image service) or host-facing-independent-v1."
      - "Select host-facing-independent-v1 only when the complete top-level extension set is the exact marker x-vat-compose-profile: host-facing-independent-v1."
      - "For host-facing-independent-v1, require two through four literal-image services, one explicit nonzero and unique host:container port per service, and optional literal environment; publish every listener only on loopback."
      - "Reject service-name DNS, dependencies, networks, volumes, build, interpolation, env-file, extra extensions, and every other Compose form before runtime launch."
      - "Use typed vat compose import/up with runtime=microvm, record the selected profile, then emit vat_docker_compose with next=docker compose -p PROJECT ps. Host-facing JSON includes profile=host-facing-independent-v1, service_name_dns=false, and host_loopback_only=true."
      - "Keep detached mode explicit: accept --wait at most once; accept --wait-timeout only with wait as positive whole seconds (default 300, maximum 1200). Start the budget after validated import and any source build, immediately before detached launch; it covers handoff plus durable VAT runner/topology observations, not Docker healthchecks, application HTTP, service DNS, or generic Compose."
      - "Pin observation to the selected profile, generation, and ticket while releasing the registry lock between polls. Ready returns one final up JSON with wait plus ready topology. Timeout retains runtime/registry and provides ps only after current-target observation; terminal/replaced/bare-deadline outcomes have no unsafe next, and degraded has no endpoint."
  - name: docker compose -f FILE -p PROJECT up -d --build [--wait [--wait-timeout SECONDS]]
    behavior:
      - "Accept only strict-single-build-v1: one literal short build-only service with no image field, one explicit nonzero host:container port, and optional literal environment."
      - "Reject build mappings, build args, custom Dockerfiles, pull/recreate/scale flags, and an already active project before starting a runtime."
      - "Capture the validated document and route it once to the typed MicroVM Compose import/build/up path so source replacement cannot bypass the strict profile and build/run use the same Apple image store."
      - "On successful non-wait up, emit only the exact VAT-built images array and cleanup_next=docker compose down plus docker image rm for that tag; literal-image up never claims image ownership. With wait, cleanup_next is emitted only in the verified ready final result."
  - name: docker compose -p PROJECT ps
    behavior:
      - "Accept only the exact no-argument ps shape; Docker --format and every other ps flag fail before observation."
      - "Route only a record carrying a known shim profile to a typed, claim-held VAT Compose observation. Generic, missing, and unknown shim provenance fail closed before inspection or topology output."
      - "Preserve the human-readable ps text and end with additive vat_docker_compose JSON carrying the known profile plus topology { phase, ready, services }, whose service entries carry name/state and an optional endpoint."
      - "Use registered service-ID order. Endpoint fields are canonical 127.0.0.1:<port> strings and appear for every service only after every expected service has unique Ready VAT-owned container_run evidence for its exact MicroVM, a nonzero loopback port, and no cleanup error."
      - "If any endpoint proof is absent, nominal ready becomes phase=degraded with ready=false and no endpoints. Inactive, starting, and stopping also expose no endpoints. This is not an app-healthcheck."
  - name: docker compose -p PROJECT logs SERVICE|down
    behavior:
      - "Route only a record carrying a known shim profile to typed VAT compose status/log/cleanup operations; generic, missing, and unknown shim provenance fail closed before inspection or mutation."
      - "Text logs preserves observed log bytes, then starts its additive VAT handoff JSON on a new line after them before terminal observed; down is terminal cleaned_up."
  - name: docker compose -p PROJECT exec -T SERVICE -- COMMAND
    behavior:
      - "Accept exact non-interactive -T only; reject default TTY, --no-tty, --index, --privileged, and other exec flags."
      - "Prove the service belongs to the known-profile project and is an exact unique ready VAT-owned Apple MicroVM before container exec; preserve observed text child bytes and numeric exit code, then start the additive vat_docker_compose handoff JSON on a new line after them with child_exit_code and next=docker compose -p PROJECT ps. This ordering makes no claim for descendants that escape the managed child."
      - "Parse and validate the Docker-facing `--` delimiter, but do not forward it: Apple Container receives `container exec CONTAINER COMMAND [ARG...]`, so an option-looking first command argument stays raw command argv."
  - name: docker compose -p PROJECT exec -T --format json SERVICE -- COMMAND | --format=json
    behavior:
      - "Accept only the two exact JSON forms: the format marker is immediately after -T, SERVICE follows it, and -- is mandatory before a non-empty command. Misordered flags, default TTY, and all other exec flags fail closed."
      - "Hold one same-read known-shim-provenance/exact-unique-ready-MicroVM snapshot through child spawn, parse and validate the literal Docker-facing -- delimiter without forwarding it, invoke Apple Container as container exec CONTAINER COMMAND [ARG...], then release the compose claim before arbitrary child wait/drain."
      - "Drain stdout/stderr concurrently and bound each serialized JSON string to 64 KiB. Emit one VAT-native vat.docker-compose.exec.v1 document with profile, child_exit_code, separate streams, truncated/utf8_lossy, runtime_invoked=true, compose_record_mutated=false, no raw child output, and no topology/endpoints; this is not Docker Compose exec output compatibility."
  - name: vat compose import|up|ps|logs|down
    behavior:
      - "Generic vat compose is a separate lifecycle surface and cannot operate a known shim-provenance record; the only transfer is an explicit inactive generic import, which clears known provenance."
      - "Generic import refuses to adopt unknown shim provenance. For an inactive unknown record, vat compose down removes only its registry project.json and leaves vat.toml intact so the user can re-import; it never touches a runtime."
      - "For unknown active provenance, generic cleanup fails closed and directs the user to a matching or newer VAT that recognizes the profile, or to the matching Docker shim."
```

## Unit Test
<!-- type: unit-test lang: yaml -->

```yaml
requirements:
  - id: T1
    text: "Pure argv translation maps documented Docker-shaped run, lifecycle, image, and copy commands to exact public Apple Container argv."
    verify: "cargo test -p vat docker_shim --lib -- --nocapture"
  - id: T2
    text: "Bare or dynamic publish ports, Engine commands, Docker-only network modes, global prune, Docker compose ps --format/filter options, default-TTY or flagged Compose exec, and lossy Compose files fail before a child process is spawned. The host-facing profile hard-rejects a missing or non-exact marker, fewer than two or more than four services, duplicate ports, non-literal images/environment, dependencies, networks, volumes, build, interpolation, and env-file."
    verify: "cargo test -p vat --test vat_docker_shim -- --nocapture"
  - id: T3
    text: "The installer cannot overwrite a foreign docker path and a translated child exit code is preserved. Generic vat compose rejects known shim provenance; generic inactive re-import clears known provenance; Docker-shaped post verbs reject generic or unknown provenance; unknown inactive registry cleanup retains vat.toml while unknown active cleanup fails closed."
    verify: "cargo test -p vat --test vat_docker_shim -- --nocapture"
  - id: T4
    text: "A deterministic fake lifecycle starts two host-facing-independent-v1 literal-image services through the shim with distinct loopback ports, proves the emitted profile/service_name_dns/host_loopback_only JSON and exact ps topology in registered service order, supports typed post verbs and exact cleanup, and complements the separately gated real dual-service host E2E rather than replacing its Apple Container evidence."
    verify: "cargo test -p vat --test vat_docker_shim compose_host_facing_independent_profile_runs_two_services_through_the_shim -- --nocapture"
  - id: T5
    text: "Typed ps topology emits canonical endpoints only for a complete unique Ready VAT-owned container_run/exact-MicroVM/loopback/nonzero/no-cleanup-error proof. Missing or duplicate evidence and every unsafe proof variant degrade ready without partial endpoints; inactive, starting, and stopping also withhold endpoints."
    verify: "cargo test -p vat docker_shim --lib -- --nocapture"
  - id: T6
    text: "Deterministic fake wait coverage requires explicit detach, one wait, and a positive 300-default/1200-max wait timeout; proves ready one-result topology, timeout retention then later recovery/down, profile/generation/ticket replacement rejection, lock release between polls, and no unsafe next for terminal/replaced/bare-deadline outcomes. Degraded wait has no endpoint and source-build cleanup_next is ready-only."
    verify: "cargo test -p vat --test vat_docker_shim -- --nocapture"
  - id: T7
    text: "Focused fake JSON exec accepts only exact --format json/equals placement and delimiter, preserves nonzero child exit plus separate bounded streams without raw replay or registry mutation, and fails closed on invalid provenance/argv."
    verify: "cargo test -p vat --test vat_docker_shim compose_host_facing_independent_profile_runs_two_services_through_the_shim -- --nocapture"
  - id: T8
    text: "Precise unit coverage proves serialized JSON stream values remain bounded under lossy and control-character expansion."
    verify: "cargo test -p vat --lib bounded_log_stream_keeps_agent_snapshots_line_and_serialized_json_bounded -- --nocapture"
  - id: T9
    text: "Stats units accept only the explicit non-streaming JSON shape, preserve native valid JSON, bound both captures at 256 KiB, and prove the single deadline plus isolated cleanup fails closed instead of replaying stdout when complete capture cannot be established."
    verify: "cargo test -p vat --lib docker_shim::tests -- --nocapture"
  - id: T10
    text: "Direct ps units accept only JSON plus optional exactly-once all, preserve one valid opaque Apple-native JSON value through ps and the two documented container aliases, reject templates/filter/quiet/positionals/unknown forms before spawn, and suppress malformed, oversized, or escaped-pipe capture without replay."
    verify: "cargo test -p vat --lib docker_shim -- --nocapture"
  - id: T11
    text: "Direct image-inventory units accept only exact JSON through images and the two documented image aliases, preserve one valid opaque Apple-native JSON value, reject every modifier/selector/positional before spawn, retain text/quiet behavior, and suppress malformed, oversized, or escaped-pipe capture without replay."
    verify: "cargo test -p vat --lib docker_shim -- --nocapture"
  - id: T12
    text: "Direct container-inspect units accept only exact JSON through inspect and the documented container inspect alias, require one safe id after one VAT-only selector, preserve valid native JSON plus nonzero child exit status, retain unformatted inspect behavior, reject object selectors/templates/second ids before spawn, and suppress malformed, oversized, or flood output without raw replay."
    verify: "cargo test -p vat --lib docker_shim -- --nocapture"
  - id: T13
    text: "Direct logs units accept only one JSON selector and one bounded tail before a final safe id through direct/container aliases, retain unformatted text behavior, normalize to Apple logs argv, and prove a VAT wrapper rather than a sixth native JSON or Docker multiplex contract. They preserve ordinary child nonzero wrapper+exit, cap both suffixes plus serialized JSON values, and fail closed without partial wrapper on timeout/setup/escaped-pipe paths."
    verify: "cargo test -p vat --lib docker_shim -- --nocapture"
  - id: T14
    text: "Direct exec units accept only direct/container aliases with one JSON selector and one 1..=1200 timeout before a safe id and mandatory Docker-facing delimiter plus raw command. They strip the delimiter for canonical Apple container exec argv, preserve raw/unformatted exec behavior, emit one bounded VAT wrapper with host-container-client-observation timeout scope, preserve ordinary child failure wrapper+exit, cap serialized suffixes, and fail closed without a partial wrapper on timeout/setup/capture paths without claiming guest-command termination."
    verify: "cargo test -p vat --lib docker_shim -- --nocapture"
  - id: T15
    text: "Direct build units accept only the strict selector/tag/context receipt grammar, strip JSON/deadline selectors before canonical public container build argv, retain successful/partial/replaced images without product cleanup, provide safe success/failure handoff, and fail closed without a receipt on timeout/setup/capture failure."
    verify: "cargo test -p vat --lib docker_shim -- --nocapture"
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-docker-shim-real-host
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "VAT_DOCKER_SHIM_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim -- --ignored --nocapture"
    assertions:
      - "A temporary shim builds a test-scoped Dockerfile image and removes it through docker image rm."
      - "A high-entropy nonce+PID owner-labeled temporary nginx docker run -d -p host:80 returns host HTTP 2xx and supports inspect/logs. Exact-label rechecks are conservative best-effort precautions and the emergency guard retains on uncertainty; Apple Container has no atomic conditional delete, so no race-free or impossible-to-misdelete cleanup claim is made."
      - "The shared/cacheable nginx image may remain; no image cleanup is claimed."
  - id: vat-docker-compose-strict-real-host
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "VAT_DOCKER_COMPOSE_SHIM_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_compose_strict_profile_contract -- --ignored --nocapture"
    assertions:
      - "A strict-single-image-v1 one-service literal nginx Compose file starts via the installed docker shim and emits the agent-facing Compose result."
      - "ps, published host HTTP, logs, and strict service-name exec -T succeed through VAT's MicroVM lifecycle without exposing the generated Apple Container name."
      - "A failing exec preserves its exact child exit code and emits a structured failure result with a runnable ps next."
      - "down emits terminal cleaned_up, removes the exact Apple Container service, and releases the published host port."
  - id: vat-docker-compose-strict-build-real-host
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "VAT_DOCKER_COMPOSE_BUILD_SHIM_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_compose_strict_build_profile_contract -- --ignored --nocapture"
    assertions:
      - "A one-service short-build Compose file builds an exact project-scoped image in Apple Container, starts it through the installed docker shim, and exposes host HTTP."
      - "ps, service-name exec -T, and logs operate over the built MicroVM service without exposing its generated Apple Container name."
      - "The public source-build up result exposes its exact image and cleanup_next; executing cleanup_next removes the exact service, host port, and image without VAT_HOME inspection or a shared-store prune."
  - id: vat-docker-compose-host-facing-independent-fake-lifecycle
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "cargo test -p vat --test vat_docker_shim compose_host_facing_independent_profile_runs_two_services_through_the_shim -- --nocapture"
    assertions:
      - "A deterministic fake runtime starts two literal-image services selected by the exact host-facing-independent-v1 marker, with two distinct loopback host ports."
      - "The successful up JSON exposes profile=host-facing-independent-v1, service_name_dns=false, and host_loopback_only=true; exact no-argument ps preserves that known profile and adds ready topology in registered docs/inspector order with canonical loopback endpoints."
      - "A typed degraded ps omits every endpoint rather than null-filling or leaking a partial topology; ps --format is unsupported and topology is not an app-healthcheck."
      - "This deterministic fixture complements the opt-in real Apple Container dual-service E2E; it does not widen that passed host evidence to service-name DNS, general Compose, Docker Engine API, or Kubernetes."
  - id: vat-docker-compose-bounded-wait-fake-lifecycle
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "cargo test -p vat --test vat_docker_shim -- --nocapture"
    assertions:
      - "The passed deterministic fake suite covers ready, timeout, later recovery/down cleanup, and down/re-import/relaunch replacement races for docker compose up -d --wait."
      - "It proves one final ready up JSON with topology, timeout runtime/registry retention, target-pinned safe ps handoff only after current observation, and no unsafe next for terminal/replaced/bare deadlines; degraded exposes no endpoint."
      - "The corresponding opt-in real Apple Container dual-service E2E is passed on this host; the fake suite remains the deterministic coverage for timeout/recovery/replacement races."
  - id: vat-docker-compose-host-facing-independent-real-host
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "RUST_TEST_THREADS=1 VAT_DOCKER_COMPOSE_INDEPENDENT_SHIM_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_compose_host_facing_independent_profile_contract -- --ignored --nocapture"
    assertions:
      - "Passed 1/1 (50 filtered) on this host in 4.54 seconds."
      - "The opt-in gated Apple Container test proves host-facing-independent-v1 up -d --wait, both loopback endpoints, one-document JSON ps/logs/exec, text logs, text exec including a no-final-newline handoff, and down cleanup of exact containers, ports, and registry."
      - "It remains evidence for this bounded profile only, not service-name DNS, general Compose, a Docker Engine API, or Kubernetes."
  - id: vat-docker-stats-fake
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "cargo test -p vat --test vat_docker_shim docker_stats -- --nocapture"
    assertions:
      - "The deterministic fake contract accepts only strict non-streaming native-JSON stats, invokes canonical Apple Container argv, preserves valid opaque native JSON and child nonzero exits, and suppresses malformed/oversized stdout."
      - "A five-second bounded observation plus isolated process-group cleanup replays stdout only after complete validated capture; an escaped pipe holder fails closed. It does not prove ownership, health, liveness, or a Docker Engine schema."
      - "Recorded validation: shared docker_shim library coverage passed 54/54. The full serial fake-shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race. Direct real-host observation passed 1/1 (50 filtered) on Apple Container 1.1.0; stats targets the temporary owner-labeled nginx container and proves one valid native JSON document only. Fake/unit tests prove byte-preservation and fail-closed details."
  - id: vat-docker-ps-json-fake
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_ps_json -- --nocapture"
    assertions:
      - "The deterministic fake contract accepts direct docker ps JSON and only the documented container ls/list aliases, normalizes to canonical Apple Container list argv, and byte-for-byte replays one validated opaque native JSON value."
      - "Templates/table output, filters, quiet plus JSON, duplicate/unknown flags, positionals, and docker container ps JSON fail before runtime; malformed, oversized, or escaped-pipe stdout fails closed under the five-second bounded isolated cleanup."
      - "Recorded validation: cargo check without default features passed; shared docker_shim library passed 54/54; focused direct ps integration passed 4/4. The full serial fake-shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race. Direct real-host observation passed 1/1 (50 filtered) on Apple Container 1.1.0; ps is a global read-only inventory smoke observation, not a targeted ownership result. It proves one valid native JSON document only; fake/unit tests prove byte-preservation and fail-closed details."
  - id: vat-docker-images-json-fake
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_images_json -- --nocapture"
    assertions:
      - "The deterministic fake contract accepts direct docker images JSON and only the documented image ls/list aliases, normalizes to canonical Apple Container image-list argv, and byte-for-byte replays one validated opaque native JSON value."
      - "Template/table/YAML/TOML output, filters, quiet, verbose, all, digests, no-trunc, positionals, duplicates, unknown flags, and -- fail before runtime; malformed, oversized, or escaped-pipe stdout fails closed under the five-second bounded isolated cleanup while text/quiet image listing remains inherited."
      - "Recorded validation: cargo check without default features passed; shared docker_shim library passed 54/54; focused docker_images_json integration passed 4/4. The full serial fake-shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race. Direct real-host observation passed 1/1 (50 filtered) on Apple Container 1.1.0; images is a global read-only inventory smoke observation, not a targeted ownership result. It proves one valid native JSON document only; fake/unit tests prove byte-preservation and fail-closed details."
  - id: vat-docker-image-inspect-json-fake
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_image_inspect_json -- --nocapture"
    assertions:
      - "The deterministic fake contract accepts only direct docker image inspect JSON with one selector before one safe opaque IMAGE, strips the selector, invokes only container image inspect IMAGE, and byte-for-byte replays one validated Apple-native JSON document."
      - "Templates, --, extra references, and every other option fail before Apple Container; a valid native document with a nonzero child exit preserves that status, while malformed, oversized, or escaped-pipe capture suppresses raw stdout under five-second bounded isolated cleanup."
      - "Recorded validation: cargo check passed; canonical cargo test -p vat --lib docker_shim -- --nocapture passed 58/58; this focused integration passed 4/4 with 1 ignored. It does not claim Docker image-inspect schema/templates/Engine API, provenance, security, registry, build-completion, readiness, or secret redaction."
  - id: vat-docker-image-inspect-json-real-host
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "RUST_TEST_THREADS=1 VAT_DOCKER_IMAGE_INSPECT_JSON_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_image_inspect_json_contract -- --ignored --nocapture"
    assertions:
      - "Passed 1/1 (61 filtered) in 1.21 seconds. The host probe validates one Apple-native JSON document for alpine:3.20 and records that VAT stripped the Docker selector before invoking only container image inspect alpine:3.20."
      - "It is bounded direct-image observation only: it does not establish Docker image-inspect schema/template/Engine API behavior, provenance, security, registry, build-completion, readiness, or secret-redaction properties."
  - id: vat-docker-inspect-json-fake
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_inspect -- --nocapture"
    assertions:
      - "The deterministic fake contract accepts direct docker inspect JSON and only the documented container inspect alias, strips the VAT-only selector, invokes canonical Apple Container inspect argv, and byte-for-byte replays one validated opaque native JSON value."
      - "--type, --size, templates/table/YAML/TOML, filters, a second id, --, and unknown flags fail before runtime; unformatted inspect remains inherited, valid JSON plus a nonzero child exit preserves status, and malformed, oversized, or flood output suppresses raw stdout under five-second bounded isolated cleanup."
      - "It is not Docker Engine inspect schema, ownership/provenance/security/image/registry/build-status, health/readiness/liveness/port-reachability evidence, or a secret-redaction guarantee. Recorded validation: cargo check without default features passed; shared docker_shim library passed 54/54; focused docker_inspect integration passed 5/5. The full serial fake-shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race. Direct real-host observation passed 1/1 (50 filtered) on Apple Container 1.1.0; inspect targets the temporary owner-labeled nginx container and proves one valid native JSON document only. Fake/unit tests prove byte-preservation and fail-closed details."
  - id: vat-docker-logs-json-fake
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_logs_json -- --nocapture"
    assertions:
      - "The deterministic fake contract accepts direct/container logs only with exact format plus bounded tail selectors before one final safe id, strips the selector, invokes canonical Apple logs argv, and emits one VAT vat.docker.logs.v1/vat_json wrapper rather than a sixth native JSON or Docker multiplex/demux schema."
      - "The wrapper carries untrusted Apple stdio, bounded diagnostic stderr, truncation/lossy flags, backend/container/requested tail/runtime/child outcome, and safe inspect next. Ordinary child failure retains wrapper+exit; follow/boot/timestamps/since/until/templates and all other modifiers reject before runtime; timeout/setup/escaped-pipe paths emit no partial wrapper after five-second plus one-second bounded cleanup with dual-stream suffix/serialized caps."
      - "Recorded validation: cargo check without default features passed; canonical cargo test -p vat --lib docker_shim -- --nocapture passed 54/54; focused docker_logs_json integration passed 6/6. The full serial fake-shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race. Direct real-host observation passed 1/1 (50 filtered) on Apple Container 1.1.0; VAT logs targets the high-entropy nonce+PID owner-labeled temporary nginx container and proves one VAT wrapper only. Exact-label rechecks are conservative best-effort precautions, the emergency guard retains on uncertainty, and Apple Container has no atomic conditional delete; this is not a race-free or impossible-to-misdelete cleanup guarantee. No shared nginx image cleanup is claimed. Fake/unit tests prove byte-preservation and fail-closed details."
  - id: vat-docker-exec-json-fake
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_exec_json -- --nocapture"
    assertions:
      - "The deterministic fake contract accepts only direct/container exec JSON with one format and one 1..=1200 timeout before a safe id, a literal Docker-facing delimiter, and a nonempty raw command; raw/unformatted exec remains generic."
      - "VAT removes the Docker-only delimiter and normalizes to Apple `container exec CONTAINER COMMAND [ARG...]`, then emits one `vat.docker.exec.v1` / `vat_json` wrapper with separate serialized-64-KiB-capped suffixes, `timeout_scope=host-container-client-observation`, and a safe inspect next."
      - "Ordinary child failure preserves wrapper+exit; timeout or setup/capture failure emits no partial wrapper. Deterministic validation passed: docker_shim library 54/54 and focused docker_exec_json 4/4. The host timeout does not claim guest command termination; no Docker Engine parity, generic runtime, Compose, or Kubernetes claim follows."
  - id: vat-docker-run-json-fake
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_run_json -- --nocapture"
    assertions:
      - "The deterministic contract accepts only direct foreground docker run JSON with flexible-order format/timeout selectors before IMAGE, rejects every caller lifecycle/network/mount/env option before Apple Container, and creates a generated high-entropy name plus independent owner label."
      - "It emits one vat.docker.run.v1/vat_json document with bounded stdout/stderr only after exact owner-label cleanup confirms absence; ordinary child nonzero retains wrapper+exit, while timeout/setup/cleanup uncertainty emits no partial wrapper and only Apple's explicit not-found diagnostic counts as absence."
      - "Passed 5 plus 1 ignored in 1.80s. The host timeout is not guest-wide termination, and this makes no crash-recovery, Docker Engine parity, or secret-redaction claim."
  - id: vat-docker-run-json-real-host
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "RUST_TEST_THREADS=1 VAT_DOCKER_RUN_JSON_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_run_json_ephemeral_contract -- --ignored --nocapture"
    assertions:
      - "Passed 1/1 (56 filtered) in 2.30s using local alpine:3.20: one foreground JSON document carries stdout/stderr markers and exact generated-container cleanup confirms absent after the run."
      - "The evidence is bounded to that owner-cleaned one-shot and does not establish guest-wide timeout termination, crash recovery, Docker Engine parity, or secret redaction."
  - id: vat-docker-build-json-fake
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "cargo test -p vat --test vat_docker_shim docker_build_json -- --nocapture"
    assertions:
      - "The deterministic contract accepts only direct build with exact format/1..=1200-timeout/tag selectors, documented optional fields before one local-directory context, and maps only supported non-selector fields to public container build argv."
      - "It emits one bounded vat.docker.build.v1/vat_json receipt after normal client completion or child nonzero, retains image lifecycle with no product auto-cleanup, safely inspects only a success, and emits no receipt on timeout/setup/capture failure."
      - "Current validation: cargo check passed; docker_shim lib 62/62; focused build suite 4 plus 1 ignored (63 filtered); native_image_owner_guard 1/1 (67 filtered). It does not claim Engine/API, provenance, ownership, readiness, security, secret-redaction, cancellation, or rollback."
  - id: vat-docker-build-json-real-host
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "RUST_TEST_THREADS=1 VAT_DOCKER_BUILD_JSON_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_build_json_receipt_contract -- --ignored --nocapture"
    assertions:
      - "Passed 1/1 (67 filtered) in 2.53 seconds. The opt-in probe proves one strict Docker-build receipt and records public Apple argv with JSON/deadline selectors stripped."
      - "Its high-entropy test tag and exact io.cclab.vat.e2e-owner label require exact native absence before build, exact label recheck before delete, and exact native absence after. This is test-only cleanup safety, not product behavior: Apple has no conditional build/delete, races are best effort and ambiguity leaks rather than authorizing cleanup."
      - "The receipt remains retained_no_auto_cleanup; this bounded host proof does not establish generic build correctness, Docker Engine/API, provenance, ownership, security, redaction, cancellation, or rollback."
  - id: vat-docker-pull-json-fake
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "cargo test -p vat --test vat_docker_shim docker_pull_json -- --nocapture"
    assertions:
      - "The deterministic contract accepts only direct pull with exact format/1..=1200-timeout selectors before one opaque image reference; it rejects empty/leading-dash/whitespace-control/URL-style `://`/leading Git-style `git@` remote forms while keeping ordinary OCI `@digest` opaque, retains raw unselected pull and docker image pull behavior, and maps only a selector-stripped request to public container image pull argv."
      - "It emits one bounded vat.docker.pull.v1/vat_json receipt only after normal client completion or child nonzero, marks the image not_owned_no_auto_cleanup with no registry management, safely inspects only a success, and emits no receipt on timeout/setup/capture/pipe failure."
      - "Current validation: cargo check passed; docker_shim lib 65/65; focused pull suite 5 plus 1 ignored (68 filtered). It does not claim Engine/API, registry auth lifecycle, provenance, digest, platform, freshness, image state, ownership, security, secret redaction, cancellation, download completion, or rollback."
  - id: vat-docker-pull-json-real-host
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "RUST_TEST_THREADS=1 VAT_DOCKER_PULL_JSON_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_pull_json_receipt_contract -- --ignored --nocapture"
    assertions:
      - "Passed 1/1 (73 filtered) in 27.14 seconds. The opt-in probe proves one strict Docker-pull receipt and records public Apple `container image pull alpine:3.20` argv with JSON/deadline selectors stripped."
      - "The E2E deliberately uses a shared/cacheable alpine image but still runs the real pull client: it neither deletes that image nor asserts ownership on success or failure. It can contact a registry or alter shared image state, so it is bounded receipt evidence rather than transfer, image-state, registry-auth, or cleanup proof."
      - "The receipt remains not_owned_no_auto_cleanup; this host proof does not establish Docker Engine/API, registry management/auth lifecycle, provenance, digest, platform, freshness, security, secret redaction, cancellation, download completion, or rollback."
  - id: vat-docker-direct-json-real-host
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-docker-command-shim
    command: "VAT_DOCKER_SHIM_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_run_published_port_contract -- --ignored --nocapture"
    assertions:
      - "Passed 1/1 (50 filtered) on Apple Container 1.1.0. The gate uses a high-entropy nonce+PID temporary nginx container and verifies io.cclab.vat.e2e-owner=<token> by inspect before cleanup. Exact-label rechecks are conservative best-effort precautions, and the emergency guard retains the container on uncertainty. Apple Container has no atomic conditional delete, so this does not claim race-free cleanup or that a misdelete is impossible; the shared/cacheable nginx image is not cleaned up."
      - "Ps/images are global read-only inventory smoke observations; inspect/stats/VAT logs and direct exec target the temporary owner-labeled nginx container. The host smoke proves one valid native JSON document or VAT wrapper only, including an exec wrapper with both stdout and stderr markers."
      - "Fake/unit tests prove byte-preservation and fail-closed details. This remains bounded direct-command evidence only: it does not establish guest-timeout termination, Docker Engine parity, generic runtime behavior, Docker multiplex/demux, broader Compose semantics, immutable image identity, or image cleanup."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/vat/src/docker_shim.rs
    action: create
    section: cli
    impl_mode: hand-written
    reason: "Raw argv multicall dispatch, fail-closed Docker-to-Apple-Container translation, five strict validated Apple-native JSON observations plus finite direct VAT-JSON logs, exec, owner-cleaned direct run, retained direct build receipts, and non-owning direct pull receipts with bounded dual-stream capture, bounded detached wait with profile/generation/ticket pinning and lock-free polling, and additive known-profile Compose ps topology JSON without Docker format compatibility."
  - path: apps/vat/src/commands/docker_shim.rs
    action: create
    section: cli
    impl_mode: hand-written
    reason: "Safe opt-in symlink installation and ownership status."
  - path: apps/vat/src/compose.rs
    action: modify
    section: cli
    impl_mode: hand-written
    reason: "Three named Docker Compose compatibility profiles, including exact host-facing marker validation, two-through-four independent literal-image services, loopback-only ports, and fail-closed topology rejection before materialization."
  - path: apps/vat/src/commands/compose.rs
    action: modify
    section: cli
    impl_mode: hand-written
    reason: "Typed Apple Container Compose lifecycle, canonical self-reexec through the multicall shim, target-pinned bounded wait observation with safe timeout retention/handoff, claim-held ps topology projection with all-or-none endpoint proof, and fail-closed generic-versus-shim provenance boundaries including unknown inactive registry-only cleanup."
  - path: apps/vat/src/main.rs
    action: modify
    section: cli
    impl_mode: hand-written
    reason: "Dispatch argv[0]=docker before VAT Clap parsing."
  - path: apps/vat/tests/vat_docker_shim.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Fake-runtime coverage for five strict native-JSON observations plus separate direct VAT-JSON logs, foreground exec, and owner-cleaned direct run wrappers; bounded ready/timeout/recovery/replacement wait behavior; host-facing two-service lifecycle/negative JSON; additive Compose ps topology and endpoint-proof gating; provenance boundaries; and opt-in real Apple Container one-service, direct-run, direct-image-inspect, and passed dual-service contracts."
  - path: apps/vat/README.md
    action: modify
    section: scenarios
    impl_mode: hand-written
    reason: "Publish the headless boundary, five strict native-JSON observations plus a separate direct VAT-JSON logs wrapper, three exact compatibility profiles, bounded detached wait and ready/timeout JSON contracts, negative/up-and-ps topology contracts, provenance limits, and honest evidence scope."
  - path: apps/vat/src/commands/llm.rs
    action: modify
    section: cli
    impl_mode: hand-written
    reason: "Teach agents five strict native-JSON observations plus the separate bounded direct VAT-JSON logs wrapper, the three-profile Compose grammar, bounded detached wait, negative contract, ps topology endpoint-proof and provenance boundaries, terminal handoff semantics, and evidence limit."
  - path: apps/vat/aw.toml
    action: modify
    section: e2e-test
    impl_mode: hand-written
    reason: "Register deterministic four-form native observation coverage plus direct VAT-JSON logs and foreground exec, three-profile/provenance/Compose-ps-topology/wait coverage, and a bounded real-host direct-observation gate; ps/images are global inventory smoke while inspect/stats/logs/exec target a temporary owner-labeled container, and fake/unit tests retain the byte-preservation/fail-closed proof."
```
