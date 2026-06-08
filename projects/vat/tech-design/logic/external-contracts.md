---
id: vat-external-contracts
summary: External contract gates for the vat README capability.
fill_sections: [e2e-test]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: host-process-execution-and-gpu-visibility
    claim: host-process-execution-and-gpu-visibility
    coverage: full
    rationale: "The EC gate verifies vat's host-process GPU claim through the README and GPU source contract."
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: agent-legible-state-and-diff-surface
    claim: agent-legible-state-and-diff-surface
    coverage: full
    rationale: "The EC gate verifies vat's agent-readable state and diff contract through the README contract."
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: copy-on-write-fork-and-snapshot-lifecycle
    claim: copy-on-write-fork-and-snapshot-lifecycle
    coverage: full
    rationale: "The EC gate verifies vat's copy-on-write fork and snapshot lifecycle through the README contract."
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: resource-isolation-boundary
    claim: resource-isolation-boundary
    coverage: full
    rationale: "The EC gate verifies vat's resource isolation boundary through the README and sandbox source contract."
---

# External Contracts: vat

## Host Process GPU Visibility EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-host-process-gpu-visibility
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: host-process-execution-and-gpu-visibility
    category: behavior
    command: "rg -n -e 'Apple GPU' -e Metal -e MPS -e MLX -e tensorflow-metal projects/vat/README.md projects/vat/src/gpu.rs"
    assertions:
      - "README/source names Apple GPU access as a host-process property"
      - "Metal, MPS, MLX, and tensorflow-metal are present in the GPU contract"
```

## Agent State And Diff EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-agent-state-and-diff-surface
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: agent-legible-state-and-diff-surface
    category: behavior
    command: "rg -n -e 'vat state' -e 'vat diff' -e '--json' -e structured projects/vat/README.md"
    assertions:
      - "README exposes vat state and vat diff"
      - "structured JSON output remains part of the agent-facing contract"
```

## Copy On Write Lifecycle EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-copy-on-write-lifecycle
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: copy-on-write-fork-and-snapshot-lifecycle
    category: behavior
    command: "rg -n -e copy-on-write -e fork -e snapshot -e clonefile -e APFS projects/vat/README.md"
    assertions:
      - "README preserves copy-on-write lifecycle language"
      - "fork, snapshot, clonefile, and APFS remain visible contract terms"
```

## Resource Isolation Boundary EC
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-resource-isolation-boundary
    capability_id: agent-native-gpu-native-dev-containers
    contract_id: resource-isolation-boundary
    category: behavior
    command: "rg -n -e sandbox -e isolation -e seatbelt projects/vat/README.md projects/vat/src/sandbox"
    assertions:
      - "vat documents resource isolation as its responsibility"
      - "sandbox and seatbelt isolation remain visible implementation surfaces"
```
