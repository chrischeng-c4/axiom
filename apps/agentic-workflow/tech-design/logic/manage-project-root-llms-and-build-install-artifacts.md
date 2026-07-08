---
id: manage-project-root-llms-and-build-install-artifacts
summary: "Manage llms.txt and build/install scripts as AW-governed project-root artifacts."
fill_sections: [scenarios, mindmap, state-machine, interaction, logic, dependency, db-model, schema, rest-api, rpc-api, async-api, cli, wireframe, component, design-token, config, manifest, runtime-image, deployment, unit-test, e2e-test]
command_refs:
  - command: aw standardize
  - command: aw standardize managed report
  - command: aw health
---

# Manage Project Root LLMS And Build Install Artifacts

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
id: aw-project-root-artifact-flow-scenarios
scenarios:
  - id: managed_scan_includes_root_artifacts
    given:
      - "an AW project root contains README.md, llms.txt, tech-design/, src/, tests/, build.sh, and install.sh"
      - "llms.txt is generated from project config, README capability path, TD root, scripts, and workspace test commands"
      - "build.sh and install.sh carry CODEGEN or HANDWRITE ownership markers tied to TD/WI references"
    when:
      - "an agent runs `aw standardize managed next <project>`"
    then:
      - "the managed coverage report includes llms.txt, build.sh, and install.sh in the in-scope artifact set"
      - "the next-action output points to a project-root artifact action when llms.txt is missing, hand-written, or stale"
      - "generated llms.txt directs agents to TD and README before implementation files"
  - id: health_reports_missing_required_root_artifacts
    given:
      - "a configured Rust project has a binary target or an installable CLI convention"
      - "the project is missing llms.txt, build.sh, or install.sh"
    when:
      - "an agent runs `aw health <project> --json`"
    then:
      - "the health report lists the missing required root artifact as a production blocker"
      - "the blocker message names the exact project-relative path"
      - "the recommended command routes back through AW standardize rather than a handwritten checklist"
  - id: health_reports_unmanaged_or_stale_llms
    given:
      - "a configured project has an existing llms.txt"
      - "llms.txt is still HANDWRITE or no longer matches the TD-first generator output"
    when:
      - "an agent runs `aw health <project>`"
    then:
      - "the health report lists llms.txt as a production blocker"
      - "the blocker tells the agent to run `aw standardize managed run <project> --non-interactive --max-ticks 1`"
      - "the generated file remains short and avoids codebase/source inventory expansion"
  - id: build_script_contract_distinguishes_debug_and_release
    given:
      - "a configured Rust binary project has an executable build.sh"
    when:
      - "an agent or skill asks AW to validate the project-root artifact contract"
    then:
      - "AW confirms build.sh exposes separate debug and release modes"
      - "AW treats a release path that installs target/debug as a contract gap"
      - "aw:build:debug and aw:build:release can dispatch through the same project-root script"
```
