---
id: "1693"
summary: Headless, one-boot Apple Container K3s sessions for one command, a bounded multi-command agent lease, and loopback Service port-forwarding with child-environment credential filtering, gated on an independently installed non-OrbStack kubectl.
fill_sections: [scenarios, cli, unit-test, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: headless-ephemeral-kubernetes-session
    coverage: partial
    rationale: "Agents need a Docker-free local Kubernetes command now, but Apple Container machine restart is not reliable enough to claim a persistent cluster backend."
---

# VAT Ephemeral Headless Local Kubernetes

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: explicit_image_then_single_command
    given:
      - "Apple Container is available"
      - "VAT's embedded systemd image is explicitly built or a supplied image exists in the same image store"
      - "an independently installed kubectl is first on PATH; an OrbStack-provided kubectl is rejected before K3s use"
    when:
      - "an agent runs vat k8s ephemeral run -- kubectl get nodes"
    then:
      - "VAT creates one generated exact-owned auto-boot machine with no home mount"
      - "VAT bootstraps the selected K3s version only through the inspect-returned backing container"
      - "host kubectl reaches one Ready node through a private copied kubeconfig"
      - "the foreground child gets KUBECONFIG, VAT_K8S_CACHE_DIR, VAT_K8S_API_SERVER, and a private HOME"
      - "the private credentials and exact owned machine are removed before VAT returns"
  - id: bootstrap_failure_collects_bounded_non_sensitive_evidence
    given:
      - "one exact owned Apple K3s machine is available but a bootstrap stage fails"
    when:
      - "VAT reports the failed one-shot or leased bootstrap"
    then:
      - "the original bootstrap error is rendered before diagnostics"
      - "VAT emits staged non-sensitive installer evidence plus exactly guest_install_log, guest_k3s_system, backing_container_logs, machine_boot_log, machine_inspect, and container_system_status"
      - "the fixed read-only diagnostics share a six-second total budget with at most one second per probe"
      - "VAT excludes private kubeconfig/cache and host credentials, does not retry bootstrap or rerun k3s --version, introduces no wrapper/recovery command, and continues existing exact cleanup"
      - "the existing 300-second bootstrap behavior is unchanged and this remains a one-boot local session, not a persistent Kubernetes promise"
  - id: interrupted_session_recovery
    given:
      - "a prior VAT process left only its non-secret session marker"
      - "the recorded PID is no longer alive"
    when:
      - "an agent runs vat k8s ephemeral cleanup"
    then:
      - "VAT reconciles only that exact machine name"
      - "VAT removes the marker only after structured absence is confirmed"
      - "a create whose client completion was never confirmed retains its marker rather than claiming that a late allocation cannot appear"
  - id: leased_agent_sequence
    given:
      - "Apple Container is available and VAT's embedded image exists"
      - "an independently installed kubectl is first on PATH; an OrbStack-provided kubectl is rejected before K3s use"
    when:
      - "an agent creates a bounded lease, then executes kubectl apply, inspect, and test as separate vat k8s session exec calls"
    then:
      - "VAT keeps one private 0700 credential/cache directory and one exact owned running machine only until explicit delete or later cleanup"
      - "each exec re-inspects the same backing container id and API endpoint and re-verifies host API access before exposing credentials"
      - "a changed backing, expired lease, uncertain create, or unconfirmed cleanup fails closed"
  - id: leased_agent_json_exec
    given:
      - "an active unexpired one-boot K3s lease has one exact owned backing machine"
    when:
      - "an agent runs vat k8s session exec --format json [--timeout seconds] <id> -- <command>"
    then:
      - "both text and JSON exec validate the active lease, exact backing id/API endpoint, private credentials, and owned host API under the private operation lock, then recheck lease expiry at the spawn boundary"
      - "omitted timeout uses remaining lease TTL; an explicit timeout is 1..=14400 seconds and cannot exceed it. Every child is placed in an owned process group, and the lock stays held through normal/deadline/interrupt cleanup and marker removal"
      - "after confirmed normal cleanup JSON emits exactly one vat.k8s.session.exec.v1 vat_json document with id, state=active, child_exit_code, separate stdout/stderr, stdout/stderr truncated and utf8-lossy indicators, api_verified=true, runtime_invoked=true, session_record_mutated=false, and a status --verify-api next command"
      - "VAT forwards the child exit, replays no raw child stream, drains stdout and stderr concurrently, and keeps each serialized JSON string value within 64 KiB by retaining a marked truncated suffix"
      - "the JSON result plus credential-validation and API-probe failures mask private credential/cache paths and do not mutate session.json; the child does receive the private credentials, so this is not a credential-free or untrusted-child boundary"
      - "a starting or live crash marker blocks later exec, delete, and cleanup fail-closed rather than claiming automatic recovery termination; the independent-kubectl leased E2E passed 1/1 (36 filtered) in 29.97s with text commands, strict JSON exec using --timeout 30, status verification, and exact delete"
  - id: leased_status_verify_api
    given:
      - "an active bounded one-boot K3s lease has no retained port-forward marker"
    when:
      - "an agent runs vat k8s session status --verify-api <id>"
    then:
      - "no-flag status remains a non-secret lease/exact-machine report without api_checked or api_state fields"
      - "the verify path acquires the private operation lock, rechecks expiry after lock, proves exact backing identity and endpoint plus private credentials, rechecks expiry immediately before one bounded API probe, and reports api_checked=true/api_state=reachable only when that exact owned API is reachable"
      - "expired leases and retained/recovery port-forward markers do not probe and report api_checked=false/api_state=not_checked; busy, unavailable, and identity-mismatched paths fail closed without changing lease state, private credentials, or marker contents"
      - "focused fake coverage passed 4/4 and the precise unit passed 1/1; the independent-kubectl leased E2E passed 1/1 (36 filtered) in 29.97s and includes status --verify-api after text and strict JSON exec. This is bounded active-lease evidence only and does not add a persistent Kubernetes or general API-status promise"
  - id: durable_boundary
    given:
      - "an agent needs retained kubeconfig, restart, storage, ingress, or multi-node behavior"
    then:
      - "VAT rejects the implied capability by not offering a keep or persistent cluster option"
      - "the agent is directed to the blocked durable microvm-k3s work instead of receiving a false success"
  - id: local_image_delivery
    given:
      - "an active lease exists and an image is already present in Apple Container's local store"
    when:
      - "an agent runs vat k8s session image load <id> <local-ref>"
    then:
      - "VAT accepts only one inspected linux/arm64 OCI variant, never an arbitrary tar path"
      - "VAT saves a private bounded OCI archive, copies it only to the exact inspected backing container, imports it into k8s.io, and verifies the canonical reference"
      - "VAT removes both host and guest archive copies before reporting success; an imagePullPolicy Never workload can use the imported image without Docker or a registry pull"
      - "The opt-in real-host local-image E2E passed 1/1 (36 filtered) in 49.73s: one already-local Apple alpine:3.20 loaded into one active lease, a pod ran it with imagePullPolicy Never and emitted its marker log, then exact session cleanup completed. This is not registry-pull generality, persistence, GUI, or Docker Engine/API evidence"
  - id: leased_service_port_forward
    given:
      - "an active unchanged K3s lease contains one literal Service"
      - "the agent needs one host assertion against that Service without VAT injecting cluster credential variables into its child environment"
      - "an independently installed kubectl is first on PATH; an OrbStack-provided kubectl is rejected before K3s use"
    when:
      - "the agent runs vat k8s session port-forward run <id> service/api 8080 -- <host-command>"
    then:
      - "VAT starts exactly one kubectl Service forward bound only to 127.0.0.1 and waits for its reported loopback port"
      - "one foreground host child receives tunnel metadata and a private HOME; VAT strips KUBECONFIG, VAT_K8S_CACHE_DIR, VAT_K8S_API_SERVER, VAT_K8S_EPHEMERAL, and VAT_HOME from its environment"
      - "the child joins the recorded authenticated kubectl process group, so normal cleanup reaps the leader and confirms ordinary cooperative, non-daemonizing descendants are gone; daemonizing or escaping that group is outside the contract"
      - "the same-UID host child is not an OS-sandboxed or adversarial security boundary; child-environment filtering is credential hygiene only"
      - "VAT writes a v2 marker with a CSPRNG private recovery identity and retains a private 0600 operation.lock opened CLOEXEC, so kubectl and the host child cannot retain the flock after a SIGKILLed VAT parent"
      - "later mutating session operations acquire that lock, rather than trusting recorded owner-PID liveness, and signal only a kubectl group leader authenticated from the v2 identity and exact loopback-forward shape; a missing, changed, or ambiguous leader retains the marker and fails closed"
      - "legacy v1 markers are never signalled: only storage cleanup is allowed after their recorded process group is already absent"
      - "before storage unlink, a durable cleaning tombstone is recorded so a torn cleanup can be retried without trusting a recycled PID"
      - "VAT removes its private forward cache and marker only after confirmed group cleanup, then reports normal cleanup or forwards the host child exit"
  - id: leased_service_port_forward_json
    given:
      - "an active unchanged K3s lease contains one literal Service and one credential-free host assertion command"
    when:
      - "the agent runs vat k8s session port-forward run --format json <id> service/api 8080 -- <host-command>"
    then:
      - "text output remains unchanged and --format json is the only JSON form; Service-only loopback forwarding, credential filtering, shared authenticated process group, and private CLOEXEC lock remain in force through cleanup"
      - "VAT silently rechecks lease validity after exact owned API proof and immediately before exact kubectl and host-child spawns, so a crossed TTL starts no tunnel"
      - "after confirmed tunnel/group cleanup and private-marker removal, VAT emits exactly one vat.k8s.session.port-forward.v1 vat_json document with child exit, separate 64 KiB serialized-capped stdout/stderr, truncation/lossy indicators, no raw replay, and a status --verify-api next command"
      - "VAT-owned setup, API, tunnel, and cleanup errors are masked; opaque credential-free child output in a successful result is preserved rather than arbitrarily redacted"
      - "if capture-reader setup is partial, VAT reaps the direct child and completes outer shared-group cleanup before joining readers; the independent-kubectl Service-forward E2E passed 1/1 (36 filtered) in 49.57s, including one Service-only loopback text and strict one-document JSON tunnel with a credential-free child, confirmed cleanup, and closed local ports; it is not a general tunnel, persistence, ingress/LB, public listener, or same-UID OS-sandbox claim"
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat k8s ephemeral image build
    behavior:
      - "Build the embedded systemd asset into Apple Container only when absent; its tag is derived from the asset text, not a verified image digest."
      - "Never starts a machine or implicitly performs this build from run."
  - name: vat k8s ephemeral run [--image ref] -- command
    behavior:
      - "Require an independently installed kubectl first on PATH and reject an OrbStack-provided binary before K3s bootstrap or host API use. This is host-tool provenance, not a GUI or Docker Engine requirement."
      - "Require one host command and an existing Apple Container machine image."
      - "Create a one-boot single-node K3s guest, verify host API access, run the child with private credentials, and preserve its normal numeric exit code only after cleanup succeeds."
      - "Remove ambient KUBECONFIG and proxy variables from setup and child execution."
      - "On a shared one-shot or leased bootstrap failure, render the root error first, then staged non-sensitive installer evidence through exactly guest_install_log, guest_k3s_system, backing_container_logs, machine_boot_log, machine_inspect, and container_system_status within a six-second total and one-second-per-probe read-only budget before existing exact cleanup. Exclude private kubeconfig/cache and host credentials; do not change the existing 300-second bootstrap behavior, retry or rerun k3s --version, or add a wrapper/recovery command."
  - name: vat k8s ephemeral cleanup [--json]
    behavior:
      - "Reconcile only VAT marker records whose PID is no longer alive."
      - "Never prefix-scan or delete an ambient Apple machine."
  - name: vat k8s session create [--image ref] [--ttl 30m]
    behavior:
      - "Create a one-boot K3s guest and retain only an opaque session id plus private 0700/0600 credentials under VAT_HOME."
      - "Accept 1m through 4h leases; output never reveals the kubeconfig path."
      - "Use the same primary-error-first bounded bootstrap diagnostics and exact cleanup as ephemeral run; diagnostic evidence does not make the lease persistent or restart-safe."
  - name: vat k8s session exec [--format json] [--timeout seconds] id -- command
    behavior:
      - "Reject expired/non-active leases and changed Apple backing IDs or API endpoints before injecting credentials."
      - "Omitted timeout uses remaining lease TTL. Explicit timeout is 1..=14400 seconds and cannot exceed remaining TTL; each text or JSON command gets an owned process group, and the operation lock remains held through group cleanup."
      - "Normal exit, deadline, or SIGINT/SIGTERM reaps the owned group before the private exec marker is removed. A starting/live crash marker blocks later exec, delete, and cleanup fail-closed; VAT does not claim to terminate an arbitrary recovered command."
      - "JSON emits one vat.k8s.session.exec.v1 vat_json document only after normal cleanup, with separate bounded stdout/stderr, child_exit_code, api_verified=true, runtime_invoked=true, session_record_mutated=false, and no raw replay. Each serialized JSON stream value is capped at 64 KiB with truncation and UTF-8-lossy indicators. Credential-validation and API-probe errors mask private credential/cache paths; the child receives private credentials, so JSON is not a credential-free or untrusted-child boundary. The leased real-host E2E passed 1/1 (36 filtered) in 29.97s with strict JSON exec using --timeout 30."
  - name: vat k8s session port-forward run [--format json] id service/name remote-port [--namespace default] [--local-port 0] -- command
    behavior:
      - "Require one unchanged active lease, one lowercase DNS Service selector, one nonzero numeric remote port, and one host command; pods and arbitrary resources are rejected."
      - "Start kubectl only with --address 127.0.0.1, wait for its loopback readiness record, and let local-port 0 select a kubectl ephemeral port."
      - "Run one foreground host child with endpoint metadata and private HOME only; strip KUBECONFIG, VAT_K8S_CACHE_DIR, VAT_K8S_API_SERVER, VAT_K8S_EPHEMERAL, and VAT_HOME from that child environment. This is credential hygiene, not a same-UID OS sandbox or adversarial-child security boundary."
      - "Join that child to the recorded authenticated kubectl process group. Normal cleanup reaps the leader and confirms ordinary cooperative, non-daemonizing descendants are gone; daemonizing or escaping the group is outside the contract."
      - "Persist a v2 CSPRNG recovery identity and a private retained 0600 CLOEXEC operation.lock. Because the lock is not inherited by kubectl or the host child, the next mutating session operation can acquire it after a SIGKILLed VAT parent and reconcile without treating owner-PID liveness as proof."
      - "Signal a recovered group only after authenticating its exact v2 leader and loopback-forward shape. Missing, changed, or ambiguous identity fails closed and retains the marker; a durable cleaning tombstone retries torn storage cleanup."
      - "Treat v1 markers as storage-only legacy records: never signal them, and remove their storage only after their recorded process group is already absent."
      - "Stop the exact forward process group and remove private marker/cache before terminal output; cleanup failure overrides child success."
      - "Text behavior remains unchanged. JSON accepts only --format json, holds the private operation lock through tunnel/group cleanup, and emits exactly one vat.k8s.session.port-forward.v1 vat_json result only after cleanup is confirmed. It preserves child exit, separately caps serialized stdout/stderr at 64 KiB, has no raw replay, and provides a status --verify-api next command."
      - "Mask VAT-owned lease/setup/API/tunnel/cleanup errors, but do not arbitrarily redact opaque credential-free child output. Recheck the lease silently after API proof and immediately before kubectl and host-child spawn; if it crosses TTL, do not start a tunnel. A partial reader setup reaps the direct child and finishes outer group cleanup before reader join."
  - name: vat k8s session image load id local-ref [--platform linux/arm64]
    behavior:
      - "Require an active unchanged lease and exactly one locally inspected linux/arm64 image variant; arbitrary tar inputs and cross-platform delivery fail closed."
      - "Save through Apple Container to a private 2 GiB-bounded OCI archive, copy/import only through the exact backing container into k8s.io, verify the canonical reference, and remove both archive copies before success."
      - "The opt-in real-host local-image E2E passed 1/1 (36 filtered) in 49.73s: one already-local Apple alpine:3.20 loaded into one lease, a pod ran it with imagePullPolicy Never and emitted its marker log, then exact session cleanup completed. It does not establish registry-pull generality, persistence, GUI, or Docker Engine/API behavior."
  - name: vat k8s session status [--verify-api] id | delete id | cleanup
    behavior:
      - "No-flag status emits only non-secret lease/machine state. `--verify-api` only accepts an active unexpired session without a retained port-forward marker; under the private operation lock it rechecks expiry, verifies exact backing identity/endpoint and private credentials, rechecks immediately before one bounded API probe, and adds api_checked=true/api_state=reachable on success."
      - "Expired or recovery-marker status remains non-probing with api_checked=false/api_state=not_checked. Busy, unavailable, and identity-mismatched paths fail closed without lease or credential mutation; no path exposes private paths. Delete proves exact machine absence before credential removal."
      - "Cleanup has no daemon behavior: it reclaims only expired leases and abandoned creates when explicitly invoked."
boundaries:
  - "Every vat k8s command requires an independently installed kubectl first on PATH and rejects an OrbStack-provided binary. On this host Homebrew /opt/homebrew/bin/kubectl is installed; independent-kubectl one-shot, leased, local-image, and Service-forward E2Es each passed 1/1 (36 filtered) in 28.38s, 29.97s, 49.73s, and 49.57s. The local-image run loaded one already-local Apple alpine:3.20 into one lease, ran a pod with imagePullPolicy Never, observed its marker log, and completed exact session cleanup; it is not registry-pull generality."
  - "No GUI or Desktop."
  - "No Docker daemon, Engine API, Docker Compose, or vat cluster backend change."
  - "No persistent/restartable or reboot-safe cluster, PVC, ingress, load balancer, multi-node, public listener, or background-tunnel promise. A bounded active lease can deliver one verified local linux/arm64 image and one foreground Service-only loopback forward, but is not durable retention."
  - "No bootstrap retry, repair, wrapper, or durability claim from diagnostics: the existing 300-second bootstrap behavior is unchanged. Failed bootstrap reports the root error first, then only the six fixed read-only labels within a six-second total and one-second-per-probe budget, excludes private kubeconfig/cache and host credentials, does not rerun k3s --version, and still performs exact cleanup."
  - "No claim that a same-UID host child is security-isolated: port-forward filters its child environment but is not an OS sandbox or adversarial-child boundary. It is a cooperative, non-daemonizing child contract; deliberately detached descendants are out of scope."
  - "The real-host leased E2E covers strict JSON exec with --timeout 30; the local-image E2E covers one already-local Apple alpine:3.20 pod with imagePullPolicy Never, a marker log, and exact session cleanup only; and the Service-forward E2E covers one Service-only loopback strict JSON tunnel with a credential-free child, confirmed cleanup, and closed local ports. None widens the one-boot, nonpersistent lease boundary into registry-pull generality, a general cluster, persistence, GUI, Docker Engine/API, ingress, public listener, or crash-safe termination claim."
```

## Unit Test
<!-- type: unit-test lang: yaml -->

```yaml
requirements:
  - id: T1
    text: "The embedded systemd asset gets a deterministic tag derived from its embedded build text."
    verify: "cargo test -p vat k8s::tests --lib -- --nocapture"
  - id: T2
    text: "Machine inspect must contain running status, exact backing container ID, and valid IP before VAT can execute guest or host commands."
    verify: "cargo test -p vat k8s::tests --lib -- --nocapture"
  - id: T3
    text: "Copied kubeconfig rewrites exactly one loopback API endpoint and private credential cleanup removes its directory."
    verify: "cargo test -p vat k8s::tests --lib -- --nocapture"
  - id: T4
    text: "A fake Apple runtime proves one-shot cleanup, a leased create → status → exec → delete lifecycle, local-image inspect → private archive → exact guest import → archive cleanup, and a Service-only loopback forward whose child environment has K3s credential variables and VAT_HOME stripped, plus exact ownership and child exit-code forwarding."
    verify: "cargo test -p vat --test vat_k8s_ephemeral -- --nocapture"
  - id: T5
    text: "Port-forward parsing accepts only literal Service selectors, nonzero remote ports, and 127.0.0.1 readiness records."
    verify: "cargo test -p vat port_forward::tests --lib -- --nocapture"
  - id: T6
    text: "Deterministic fake tests prove a TERM-ignoring same-group host descendant is gone before cleanup=confirmed, and prove a SIGKILLed VAT parent is recovered through an exec-wrapper kubectl once a later operation acquires the released CLOEXEC flock. They also cover v2 stale-marker and v1 already-absent-group storage cleanup. The recovery contract deliberately retains an unauthenticated leader, and a cleaning tombstone makes torn unlink retryable."
    verify: "cargo test -p vat --test vat_k8s_ephemeral -- --nocapture"
  - id: T7
    text: "The passed deterministic fake bootstrap regression keeps the primary error before staged non-sensitive diagnostics, reports exactly guest_install_log, guest_k3s_system, backing_container_logs, machine_boot_log, machine_inspect, and container_system_status under the six-second total/one-second-probe budget, excludes private credentials, does not rerun k3s --version, and completes exact cleanup."
    verify: "cargo test -p vat --test vat_k8s_ephemeral -- --nocapture"
  - id: T8
    text: "Focused fake API-status coverage proves reachable exact API output, non-probing recovery and expired state, and fail-closed busy/unavailable/identity mismatch without lease or credential mutation."
    verify: "cargo test -p vat --test vat_k8s_ephemeral leased_session_status_verify_api -- --nocapture"
  - id: T9
    text: "A precise unit locks the api_checked/api_state vocabulary and expiry-recheck boundary."
    verify: "cargo test -p vat --lib api_status_expiry_recheck_rejects_a_lease_that_crossed_its_deadline -- --nocapture"
  - id: T10
    text: "Deterministic fake JSON exec emits one vat.k8s.session.exec.v1 agent document with separate streams, child-exit preservation, no raw replay, no private-path rendering, and no session marker mutation; it refuses to spawn a credentialed child when the owned API probe crosses the lease expiry and masks private paths when credentials or the API probe fail."
    verify: "cargo test -p vat --test vat_k8s_ephemeral leased_session_exec -- --nocapture"
  - id: T11
    text: "A precise unit proves the newest UTF-8-safe suffix of a lossy/control-heavy JSON exec stream remains within the 64 KiB serialized JSON-value cap and is marked truncated."
    verify: "cargo test -p vat --lib bounded_stream_preserves_the_latest_serializable_agent_snapshot -- --nocapture"
  - id: T12
    text: "Deterministic fake JSON port-forward covers one post-cleanup vat.k8s.session.port-forward.v1 result with child-exit preservation, no raw replay, separate bounded streams, masked VAT-owned failures, silent post-API/spawn TTL checks, and no tunnel after expiry."
    verify: "cargo test -p vat --test vat_k8s_ephemeral leased_session_port_forward_json -- --nocapture"
  - id: T13
    text: "The JSON tunnel capture unit preserves a latest serializable 64 KiB stream suffix, and partial reader setup reaps the direct child then allows outer shared-group cleanup before reader join."
    verify: "cargo test -p vat --lib port_forward_json -- --nocapture"
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-headless-ephemeral-k8s-bootstrap-diagnostics
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-ephemeral-kubernetes-session
    command: "cargo test -p vat --test vat_k8s_ephemeral -- --nocapture"
    assertions:
      - "The passed deterministic fake regression keeps the bootstrap root error primary, then emits staged non-sensitive installer/guest/machine diagnostics with exactly guest_install_log, guest_k3s_system, backing_container_logs, machine_boot_log, machine_inspect, and container_system_status."
      - "The diagnostics are fixed read-only probes under a six-second total and one-second-per-probe budget; they exclude private kubeconfig/cache and host credentials, do not change the existing 300-second bootstrap behavior, do not rerun k3s --version or add a wrapper/recovery command, and exact cleanup still runs."
  - id: vat-headless-leased-k8s-json-exec-fake
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-ephemeral-kubernetes-session
    command: "cargo test -p vat --test vat_k8s_ephemeral leased_session_exec -- --nocapture"
    assertions:
      - "The fake runtime proves the JSON leased exec result has one VAT-owned document, separate bounded streams, a preserved child exit code, no raw replay, and no session marker mutation; credential-validation and API-probe failures mask private paths."
      - "A fake owned-API probe that crosses the lease deadline does not spawn the credentialed child. The independent-kubectl leased real-host E2E passed 1/1 (36 filtered) in 29.97s and includes strict JSON exec with --timeout 30."
  - id: vat-headless-leased-k8s-port-forward-json-fake
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-ephemeral-kubernetes-session
    command: "cargo test -p vat --test vat_k8s_ephemeral leased_session_port_forward_json -- --nocapture"
    assertions:
      - "The fake runtime proves text stays separate while JSON returns one vat.k8s.session.port-forward.v1 only after exact tunnel/group cleanup, preserving the host child exit and separately bounded stream snapshots without raw replay."
      - "VAT masks its own setup/API/tunnel/cleanup errors, preserves opaque credential-free child output in a successful result, refuses to open a tunnel when the lease crosses expiry after API proof, and cleans direct/outer children before readers join after partial setup failure."
      - "Focused deterministic filter passed 7/7. The independent-kubectl Service-forward E2E passed 1/1 (36 filtered) in 49.57s and includes the strict JSON tunnel only for one Service-only loopback session."
  - id: vat-headless-ephemeral-k8s-real-host
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-ephemeral-kubernetes-session
    command: "VAT_K8S_EPHEMERAL_E2E_REQUIRED=1 cargo test -p vat --test vat_k8s_ephemeral apple_container_k3s_session_exposes_host_api_then_cleans_up -- --ignored --nocapture"
    assertions:
      - "An independently installed non-OrbStack kubectl is first on PATH; an OrbStack-provided binary is rejected before the K3s command runs."
      - "The public command's systemd image is present in Apple Container."
      - "Host kubectl reads exactly one Ready K3s node through the temporary 0600 kubeconfig."
      - "The terminal result's exact machine name returns Apple Container's not-found result after return."
      - "A pass is only a one-boot agent-session proof, never a durable microvm-k3s claim."
  - id: vat-headless-leased-k8s-real-host
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-ephemeral-kubernetes-session
    command: "VAT_K8S_SESSION_E2E_REQUIRED=1 cargo test -p vat --test vat_k8s_ephemeral apple_container_k3s_leased_session_supports_multiple_host_commands_then_deletes -- --ignored --nocapture"
    assertions:
      - "Two independent host kubectl commands use one active private lease."
      - "Explicit delete confirms exact Apple machine absence and removes the credential directory."
      - "The proof does not claim Apple-machine restart, reboot persistence, or a durable cluster backend."
  - id: vat-headless-leased-k8s-local-image-real-host
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-ephemeral-kubernetes-session
    command: "RUST_TEST_THREADS=1 VAT_K8S_LOCAL_IMAGE_E2E_REQUIRED=1 cargo test -p vat --test vat_k8s_ephemeral apple_container_k3s_lease_imports_local_image_without_registry_pull -- --ignored --nocapture"
    assertions:
      - "Passed 1/1 (36 filtered) in 49.73s: an already-local Apple alpine:3.20 image is inspected, privately delivered into one active K3s lease, and reported with an OCI descriptor digest."
      - "The one fixture pod uses imagePullPolicy Never, completes, and emits its marker log; this proves the imported local image for that pod only, not registry-pull generality."
      - "Explicit delete confirms exact Apple machine and private session storage cleanup. This is not persistence, GUI, or Docker Engine/API evidence."
  - id: vat-headless-leased-k8s-port-forward-real-host
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: headless-ephemeral-kubernetes-session
    command: "VAT_K8S_PORT_FORWARD_E2E_REQUIRED=1 cargo test -p vat --test vat_k8s_ephemeral apple_container_k3s_lease_port_forwards_local_service_to_one_credential_free_host_child -- --ignored --nocapture"
    assertions:
      - "Independent-kubectl real-host E2E passed 1/1 (36 filtered) in 49.57s. A local alpine fixture was loaded into the active K3s lease; because BusyBox lacks `httpd`, an in-pod HTTP probe verified the fixture before its Service endpoint responded through one VAT-owned 127.0.0.1 text forward and the strict one-document JSON tunnel."
      - "One credential-free host curl child proves VAT supplied endpoint metadata but did not inject kubeconfig/cache/API variables or VAT_HOME. Its terminal record begins on a new line after child output."
      - "The terminal result confirms forward cleanup, the selected loopback port is closed afterward, and the lease remains active only until explicit delete confirms exact machine cleanup. This bounded result is not a same-UID OS-isolation claim, daemonized-child claim, persistence, ingress, public bind, background-proxy, or general Kubernetes claim."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/vat/assets/k8s/ephemeral-machine/Dockerfile
    action: create
    section: cli
    impl_mode: hand-written
    reason: "Embed the known Phase-0-proven systemd machine asset in released VAT binaries."
  - path: apps/vat/src/commands/k8s.rs
    action: create
    section: cli
    impl_mode: hand-written
    reason: "Own bounded Apple-machine creation, backing identity checks, K3s bootstrap with primary-error-first bounded non-sensitive diagnostics, one-shot/private leased credential injection, JSON-session-exec and JSON-port-forward dispatch, verified local-image delivery, Service-forward dispatch, exact cleanup, and stale-marker/expired-lease reconciliation."
  - path: apps/vat/src/commands/k8s/port_forward.rs
    action: create
    section: cli
    impl_mode: hand-written
    reason: "Own one foreground Service-only 127.0.0.1 forward, text and post-cleanup JSON result orchestration, child-environment credential filtering (not OS isolation), authenticated shared-process-group cleanup for cooperative children, v2 CSPRNG fail-closed recovery, v1 storage-only compatibility, CLOEXEC lock recovery after SIGKILL, and durable cleanup tombstones without a background proxy."
  - path: apps/vat/src/commands/k8s/port_forward_json.rs
    action: create
    section: cli
    impl_mode: hand-written
    reason: "Own concurrent bounded agent capture for the credential-free port-forward host child, including serialized stream caps and post-cleanup-only reader joins so inherited pipes cannot block group cleanup."
  - path: apps/vat/src/commands/k8s/session_exec.rs
    action: create
    section: cli
    impl_mode: hand-written
    reason: "Own optional agent JSON capture for one credentialed leased-session host command: exact lease/backing/API/private-credential proof through spawn, remaining-TTL/default and bounded explicit timeout handling, owned-group cleanup under the private lock, crash-marker fail-closed behavior, bounded concurrent stream capture, one VAT-native result document, private-path masking, and no session-record mutation. The independent-kubectl leased E2E passed 1/1 (36 filtered) in 29.97s with text commands, strict JSON exec using --timeout 30, status verification, and exact delete; it does not establish crash-safe termination."
  - path: apps/vat/src/cli.rs
    action: modify
    section: cli
    impl_mode: hand-written
    reason: "Expose explicit one-shot, leased-session, lease-local-image, Service-only port-forward including `--format json`, and optional `session exec --format json` command hierarchies."
  - path: apps/vat/tests/vat_k8s_ephemeral.rs
    action: create
    section: unit-test
    impl_mode: hand-written
    reason: "Exercise public one-shot/lease/local-image/Service-forward behavior plus JSON session exec and JSON port-forward against a fake runtime, including passed bootstrap-diagnostic regression, TTL/cleanup/reader-failure safety, TERM-ignoring descendant cleanup, and exec-wrapper SIGKILL recovery. Independent-kubectl one-shot, leased, local-image, and Service-forward E2Es passed 1/1 (36 filtered) in 28.38s, 29.97s, 49.73s, and 49.57s; the local-image run loads one already-local Apple alpine:3.20 into one lease, runs a pod with imagePullPolicy Never, observes its marker log, then completes exact session cleanup—not registry-pull generality—while the leased run covers strict JSON exec with --timeout 30 and exact delete, and the Service-forward run covers strict JSON tunnel cleanup and closed local ports. These remain bounded one-guest proofs, not persistence, GUI, Docker Engine/API, a general tunnel, OS isolation, or crash-safe termination."
  - path: apps/vat/README.md
    action: modify
    section: scenarios
    impl_mode: hand-written
    reason: "Publish one-shot, bounded-lease, local-image, bounded JSON session exec and post-cleanup JSON Service-forward behavior, bootstrap diagnostics, and hardened child-environment-filtered loopback forwarding. Independent-kubectl leased and Service-forward E2Es cover strict JSON exec and one strict Service-only JSON tunnel; do not advertise persistent local Kubernetes, OS security isolation, daemonized-child support, a general tunnel, or crash-safe termination."
```
