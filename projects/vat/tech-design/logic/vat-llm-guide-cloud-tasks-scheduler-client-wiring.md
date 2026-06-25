---
id: vat-llm-guide-cloud-tasks-scheduler-client-wiring
summary: Extend the `vat llm` agent usage guide and the README emulator section to document how to wire a Cloud Tasks / Cloud Scheduler client at vat's built-in REST emulator — those SDKs do not auto-read the emulator host var and default to gRPC, so a factory must force the REST transport, an `http://$HOST` endpoint, and anonymous credentials.
fill_sections: [scenarios, cli, e2e-test, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: agent-legible-state-and-diff-surface
    claim: agent-legible-state-and-diff-surface
    coverage: partial
    rationale: "Closes a usability gap in vat's agent-facing usage contract: a consuming agent could not reliably connect a Cloud Tasks / Cloud Scheduler client to vat's built-in REST emulator because, unlike pubsub/firebase-auth/firestore/GCS, those SDKs do not auto-read the emulator host var and default to gRPC."
---

# Vat LLM Guide: Cloud Tasks / Cloud Scheduler Client Wiring

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
id: vat-llm-guide-cloud-tasks-scheduler-client-wiring-scenarios
scenarios:
  - id: llm_guide_documents_cloud_tasks_client_wiring
    given:
      - "an agent needs to point a Cloud Tasks or Cloud Scheduler client at vat's built-in emulator"
    when:
      - "the agent runs `vat llm`"
    then:
      - "the guide states that the cloud-tasks and cloud-scheduler SDKs do not auto-read CLOUD_TASKS_EMULATOR_HOST / CLOUD_SCHEDULER_EMULATOR_HOST and default to gRPC while vat serves REST"
      - "the guide says a plain host or DNS redirect does not work for these two"
      - "the guide instructs building the client through one factory that, when the host var is present, forces the REST transport, an http://$HOST endpoint, and anonymous credentials"
      - "the guide gives a concrete Python and Node example and points at the direct-REST alternative in tests/vat_emulator_tasks.rs"
  - id: llm_guide_keeps_auto_host_sdks_distinct
    given:
      - "an agent reads the emulator wiring guidance"
    when:
      - "the agent runs `vat llm`"
    then:
      - "the guide keeps pubsub, firebase-auth, firestore, and GCS as auto-host-reading SDKs that need no override"
```

## CLI
<!-- type: cli lang: yaml -->

```yaml
commands:
  - name: vat llm
    behavior:
      - "Continue to print the stable markdown agent usage guide."
      - "Include a client-wiring note for the cloud-tasks and cloud-scheduler presets: their official SDKs do not auto-read the emulator host var and default to gRPC, vat serves REST, so build the client through a factory that forces REST transport, an http://$HOST endpoint, and anonymous credentials."
      - "Distinguish the auto-host-reading SDKs (pubsub, firebase-auth, firestore, GCS) that need no override."
      - "Point agents at the direct-REST alternative in tests/vat_emulator_tasks.rs."
```

## E2E Test
<!-- type: e2e-test lang: yaml -->

```yaml
e2e_tests:
  - id: vat-llm-guide-cloud-tasks-scheduler-client-wiring
    name: "vat llm guide documents cloud-tasks client wiring"
    capability_id: agent-native-gpu-native-dev-containers
    claim_id: agent-legible-state-and-diff-surface
    contract_id: agent-legible-state-and-diff-surface
    category: behavior
    command: "cargo test -p vat llm_guide_mentions_core_agent_contract -- --nocapture"
    assertions:
      - "`vat llm` exits successfully."
      - "The guide still mentions vat.toml runner mode, direct command mode, and state/diff/logs evidence commands."
      - "The guide mentions the cloud-tasks / cloud-scheduler client-wiring factory (REST transport + http endpoint + anonymous credentials)."
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/commands/llm.rs
    action: modify
    section: scenarios
    impl_mode: hand-written
    reason: "Add the cloud-tasks / cloud-scheduler client-wiring note to the hand-written `vat llm` guide string."
  - path: projects/vat/README.md
    action: modify
    section: cli
    impl_mode: hand-written
    reason: "Mirror the client-wiring note in the README emulator section for operator/agent parity."
  - path: projects/vat/tests/vat_toml_runner.rs
    action: validate
    section: e2e-test
    impl_mode: hand-written
    reason: "The existing `vat llm` guide smoke test verifies the new client-wiring note is present."
```
