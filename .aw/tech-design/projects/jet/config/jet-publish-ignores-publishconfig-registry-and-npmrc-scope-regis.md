---
id: projects-jet-config-jet-publish-ignores-publishconfig-registry-and-npmrc-scope-regis-md
fill_sections: [logic, unit-test, changes]
capability_refs:
  - id: library-build-publishing
    role: primary
    gap: library-publishing-readiness
    claim: publish-and-private-registry
    coverage: partial
    rationale: "jet publish must resolve the target registry from package publishConfig.registry or .npmrc scoped registry before defaulting to npmjs.org."
---

# jet publish: Registry Resolution Honors Package and Scope Configuration

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-publish-registry-selection-contract
entry: prepare_publish
nodes:
  read_manifest: { kind: start, label: "Read package.json for publish package" }
  identity: { kind: process, label: "Require name/version before routing" }
  publish_config: { kind: decision, label: "publishConfig.registry present?" }
  scoped_npmrc: { kind: decision, label: ".npmrc has @scope:registry for package?" }
  default_registry: { kind: process, label: "Use default npm registry" }
  registry: { kind: process, label: "Use selected registry for auth and upload/dry-run" }
  auth: { kind: process, label: "Require token for selected registry" }
  preview_or_put: { kind: terminal, label: "Dry-run reports same registry real publish would PUT" }
edges:
  - { from: read_manifest, to: identity }
  - { from: identity, to: publish_config }
  - { from: publish_config, to: registry, label: "yes" }
  - { from: publish_config, to: scoped_npmrc, label: "no" }
  - { from: scoped_npmrc, to: registry, label: "yes" }
  - { from: scoped_npmrc, to: default_registry, label: "no" }
  - { from: default_registry, to: registry }
  - { from: registry, to: auth }
  - { from: auth, to: preview_or_put }
---
flowchart TD
    read_manifest([Read package.json]) --> identity[Require package name/version]
    identity --> publish_config{publishConfig.registry?}
    publish_config -->|yes| registry[Selected registry]
    publish_config -->|no| scoped_npmrc{Scoped .npmrc registry?}
    scoped_npmrc -->|yes| registry
    scoped_npmrc -->|no| default_registry[Default registry.npmjs.org]
    default_registry --> registry
    registry --> auth[Require token for selected registry]
    auth --> preview_or_put([Dry-run report or real PUT uses selected registry])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-publish-registry-selection-tests
requirements:
  R1:
    text: "publishConfig.registry on package.json overrides npmrc default/scoped registry selection for publish and dry-run."
    risk: high
    verify: unit
  R2:
    text: ".npmrc @scope:registry remains the fallback for scoped packages when publishConfig.registry is absent."
    risk: high
    verify: unit
  R3:
    text: "Dry-run and real publish share the same selected registry and auth-token lookup path."
    risk: high
    verify: unit
---
requirementDiagram
requirement R1 {
  id: R1
  text: "publishConfig.registry wins"
  risk: High
  verifymethod: Test
}
requirement R2 {
  id: R2
  text: "Scoped npmrc fallback remains"
  risk: High
  verifymethod: Test
}
requirement R3 {
  id: R3
  text: "Dry-run and publish share registry"
  risk: High
  verifymethod: Test
}
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/pkg_manager/publish.rs"
    action: modify
    section: logic
    description: |
      Resolve the publish registry from package.json publishConfig.registry
      before falling back to .npmrc scoped/default registry selection, and keep
      the selected registry in the shared prepare_publish path used by dry-run
      and real publish.
    impl_mode: hand-written
  - path: "projects/jet/tests/publish/library_publish_e2e.rs"
    action: modify
    section: unit-test
    description: |
      Add mock-registry coverage proving publishConfig.registry drives dry-run
      and real publish routing even when .npmrc points the same scope elsewhere.
    impl_mode: hand-written
  - path: "projects/jet/src/pkg_manager/npmrc.rs"
    action: modify
    section: unit-test
    description: |
      Keep scoped .npmrc registry fallback covered so publishConfig precedence
      does not regress existing scoped package routing.
    impl_mode: hand-written
```
