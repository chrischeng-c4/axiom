---
id: tape-operator-k8s-dockerfile-cli
summary: >
  Deploy/ops surface for apps/tape (WI #1328, epic #1324), mirroring relay's
  WI #1208 slice against the shared libs/operator scaffold. Adds an optional
  `operator` cargo feature (kube/k8s-openapi/schemars/operator, unconditional
  serde_yaml) and a TapeSpec CRD (group tape.dev, v1alpha1, kind Tape) that
  flattens operator::ClusterSpec with shardCount pinned to 1 (tape is a
  single raft group per the Primary Replicas capability row) plus tape's own
  knobs (storage, storageClass, graceSecs, logLevel, auth, tokensSecret).
  Render always produces a StatefulSet (per-pod /data PVC for raft hard
  state + the applied-index marker) + headless/client Services + PDB via the
  shared operator::render toolkit, hardened with tape's health probes
  (/healthz /readyz) and nonroot security context; the opt-in
  TAPE_AUTH/TAPE_TOKEN_REGISTRY_FILE Secret wiring follows relay/lumen's
  pattern. `tape k8s crd/operator/instance render` + `tape k8s operator run`
  (behind `--features operator`) and `tape dockerfile render --variant
  source|release` are new CLI subcommands on the existing tape binary
  (Append/Replay/Checkpoint/Serve/Spec/Llm/Upgrade/Issue stay unchanged).
  Dockerfile/Dockerfile.release are checked-in fixtures the render verb
  reproduces byte-for-byte (relay/keep pattern). No live kind cluster is
  available in this environment; verification is offline (compiled-binary
  CLI tests asserting parseable YAML, structural-schema-safe CRD, and
  byte-identical Dockerfile fixtures), scoped down from relay's kind-based
  smoke script.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: tape-operator-k8s-dockerfile-flow
entry: cli
nodes:
  cli:
    kind: start
    label: "tape binary gains new top-level subcommands alongside the existing Append/Replay/Checkpoint/Serve/Spec/Llm/Upgrade/Issue: K8s(crd|operator|instance) and Dockerfile(render)"
  route:
    kind: decision
    label: "Which subcommand?"
  crd:
    kind: process
    label: "tape k8s crd render: emits the TapeSpec CustomResourceDefinition YAML (default build: checked-in fixture via include_str!; --features operator: generated live from TapeSpec::crd() + Kubernetes structural-schema uint normalization)"
  opgate:
    kind: decision
    label: "tape k8s operator <render|run>: render is always offline (checked-in RBAC/Deployment fixtures, namespace-substituted); run needs --features operator"
  oprender:
    kind: process
    label: "operator render: namespace/ServiceAccount/ClusterRole/ClusterRoleBinding/Deployment YAML with namespace substituted"
  oprun_ok:
    kind: process
    label: "operator build: run drives the shared libs/operator controller (operator::run::<Tape>()) -- watches Tape CRs cluster-wide, server-side-applies render() output, writes status via ManagedService"
  oprun_fail:
    kind: terminal
    label: "default build: run exits nonzero with a rebuild-with --features operator hint"
  instance:
    kind: process
    label: "tape k8s instance render --profile dev|staging|prod|template: renders a namespaced kind: Tape CR; prod pins shardCount=1, replicasPerShard=3, voterCount=3, auth=required, tokensSecret set (tape is a single raft group -- Primary Replicas capability)"
  render_children:
    kind: process
    label: "operator::render toolkit (feature-gated): ServiceAccount + sharded_statefulset (shardCount pinned 1, /data PVC for raft hard state + applied-index marker, TAPE_BIND/TAPE_DATA_DIR/TAPE_GRACE_SECS env, opt-in TAPE_AUTH/TAPE_TOKEN_REGISTRY_FILE Secret mount) + headless/client Services + PDB, hardened with /healthz /readyz probes and a nonroot security context"
  dockerfile:
    kind: process
    label: "tape dockerfile render --variant source|release [--version] [--out]: strips ownership markers from the checked-in Dockerfile / Dockerfile.release fixture (include_str!) and substitutes the release ARG/tag when --version is given -- render is the in-binary form of the fixture, byte-identical by construction"
  done:
    kind: terminal
    label: "YAML/Dockerfile text written to --out or stdout; no server, no cluster I/O outside operator run"
edges:
  - { from: cli, to: route }
  - { from: route, to: crd, label: "k8s crd render" }
  - { from: route, to: opgate, label: "k8s operator" }
  - { from: route, to: instance, label: "k8s instance render" }
  - { from: route, to: dockerfile, label: "dockerfile render" }
  - { from: opgate, to: oprender, label: "render" }
  - { from: opgate, to: oprun_ok, label: "run, operator feature on" }
  - { from: opgate, to: oprun_fail, label: "run, operator feature off" }
  - { from: instance, to: render_children, label: "operator build reconciles the rendered CR" }
  - { from: crd, to: done }
  - { from: oprender, to: done }
  - { from: oprun_ok, to: done }
  - { from: render_children, to: done }
  - { from: dockerfile, to: done }
---
flowchart TD
    cli([tape binary: new K8s and Dockerfile subcommands]) --> route{Which subcommand?}
    route -->|k8s crd render| crd[TapeSpec CRD YAML: fixture in default build, generated in operator build]
    route -->|k8s operator| opgate{render or run?}
    route -->|k8s instance render| instance[namespaced kind: Tape CR, profile dev/staging/prod/template]
    route -->|dockerfile render| dockerfile[strip markers from checked-in Dockerfile fixture, substitute version]
    opgate -->|render| oprender[namespace/RBAC/Deployment YAML, namespace substituted]
    opgate -->|run, operator feature on| oprun_ok[operator::run over Tape: watch, SSA render, status]
    opgate -->|run, operator feature off| oprun_fail([nonzero exit, rebuild hint])
    instance --> render_children[operator::render toolkit: StatefulSet + Services + PDB, shardCount pinned 1]
    crd --> done([YAML/Dockerfile to stdout or --out])
    oprender --> done
    oprun_ok --> done
    render_children --> done
    dockerfile --> done
```
