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
