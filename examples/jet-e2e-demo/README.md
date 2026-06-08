# Jet E2E Demo

Minimal product-flow E2E demo for `jet e2e run` and `jet e2e open`.

The spec models the Cue Artifact Studio flow in a real Jet Browser page without starting a backend:

1. create a project
2. promote a work item through implementation
3. publish an artifact

Run agent/CI mode:

```bash
cargo run -p jet -- e2e run examples/jet-e2e-demo/cue-artifact-studio.spec.js \
  --evidence-dir examples/jet-e2e-demo/test-results/run \
  --json
```

Export human review mode without launching the desktop-style review shell:

```bash
cargo run -p jet -- e2e open examples/jet-e2e-demo/cue-artifact-studio.spec.js \
  --evidence-dir examples/jet-e2e-demo/test-results/open \
  --no-open
```

Run human review mode. This opens a desktop-style review shell for the case list,
command log, and replay controls while the AUT runs in a separate visible
controlled Jet Browser target:

```bash
cargo run -p jet -- e2e open examples/jet-e2e-demo/cue-artifact-studio.spec.js \
  --evidence-dir examples/jet-e2e-demo/test-results/open-live
```

After `open --no-open`, inspect:

```text
examples/jet-e2e-demo/test-results/open/open-runner-shell/index.html
```
