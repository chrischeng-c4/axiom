---
id: semantic-jet-e2e
summary: Semantic coverage for "projects/jet/src/e2e"
fill_sections: [schema, tests, changes]
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "Traceability repair: this existing Jet TD/source edge supports the aggregate production replacement capability."
---

# Semantic TD: jet/e2e

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "jet/e2e"
  source_group: "projects/jet/src/e2e"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/jet/src/e2e/selectors.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method", "test_case"]
        symbols:
          - name: "SELECTOR_EVIDENCE_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "SelectorKind"
            kind: "enum"
            public: true
          - name: "Selector"
            kind: "struct"
            public: true
          - name: "css"
            kind: "function"
            public: true
          - name: "role"
            kind: "function"
            public: true
          - name: "text"
            kind: "function"
            public: true
          - name: "test_id"
            kind: "function"
            public: true
          - name: "with_description"
            kind: "function"
            public: true
          - name: "SelectorMissReason"
            kind: "enum"
            public: true
          - name: "SelectorResolution"
            kind: "enum"
            public: true
          - name: "hit"
            kind: "function"
            public: true
          - name: "miss"
            kind: "function"
            public: true
          - name: "with_detail"
            kind: "function"
            public: true
          - name: "is_hit"
            kind: "function"
            public: true
          - name: "SelectorEvidence"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "is_hit"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/dom_snapshot.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "enum_model", "service_method"]
        symbols:
          - name: "DOM_SNAPSHOT_ARTIFACT_KIND"
            kind: "constant"
            public: true
          - name: "DomSnapshotMode"
            kind: "enum"
            public: true
          - name: "should_capture"
            kind: "function"
            public: true
          - name: "register_step_dom_snapshot"
            kind: "function"
            public: true
          - name: "maybe_register_failed_step_dom_snapshot"
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
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/actionability.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "ACTIONABILITY_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "ActionabilityReason"
            kind: "enum"
            public: true
          - name: "ElementProbe"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "ActionabilityResult"
            kind: "enum"
            public: true
          - name: "is_actionable"
            kind: "function"
            public: true
          - name: "check_actionability"
            kind: "function"
            public: true
          - name: "check_visibility"
            kind: "function"
            public: false
          - name: "check_enabled"
            kind: "function"
            public: false
          - name: "check_hit_target"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/browser_session.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "BROWSER_SESSION_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "BrowserDriver"
            kind: "enum"
            public: true
          - name: "SessionAnchor"
            kind: "enum"
            public: true
          - name: "SessionState"
            kind: "enum"
            public: true
          - name: "SessionEvent"
            kind: "enum"
            public: true
          - name: "BrowserSessionRequest"
            kind: "struct"
            public: true
          - name: "run_mode_default"
            kind: "function"
            public: true
          - name: "BrowserSessionRecord"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "launch"
            kind: "function"
            public: true
          - name: "ready"
            kind: "function"
            public: true
          - name: "enter_case"
            kind: "function"
            public: true
          - name: "finish_case"
            kind: "function"
            public: true
          - name: "close"
            kind: "function"
            public: true
          - name: "fail"
            kind: "function"
            public: true
          - name: "is_terminal"
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
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/assertion_diff.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "ASSERTION_DIFF_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "DiffLine"
            kind: "enum"
            public: true
          - name: "marker"
            kind: "function"
            public: true
          - name: "text"
            kind: "function"
            public: true
          - name: "AssertionDiff"
            kind: "struct"
            public: true
          - name: "from_pair"
            kind: "function"
            public: true
          - name: "to_text"
            kind: "function"
            public: true
          - name: "is_unchanged"
            kind: "function"
            public: true
          - name: "compute_line_diff"
            kind: "function"
            public: false
          - name: "longest_common_subseq"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/clock.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "CLOCK_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "ClockMode"
            kind: "enum"
            public: true
          - name: "ClockControl"
            kind: "struct"
            public: true
          - name: "ClockEvent"
            kind: "enum"
            public: true
          - name: "frozen_at"
            kind: "function"
            public: true
          - name: "live"
            kind: "function"
            public: true
          - name: "advance"
            kind: "function"
            public: true
          - name: "set_time"
            kind: "function"
            public: true
          - name: "release"
            kind: "function"
            public: true
          - name: "to_init_script"
            kind: "function"
            public: true
          - name: "ClockEvidence"
            kind: "struct"
            public: true
          - name: "from_control"
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
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/discovery.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "E2E_CASE_FILE_SUFFIXES"
            kind: "constant"
            public: true
          - name: "E2E_DISCOVERY_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "E2eCase"
            kind: "struct"
            public: true
          - name: "DiscoveryManifest"
            kind: "struct"
            public: true
          - name: "TagFilter"
            kind: "struct"
            public: true
          - name: "matches"
            kind: "function"
            public: true
          - name: "discover"
            kind: "function"
            public: true
          - name: "filter"
            kind: "function"
            public: true
          - name: "is_case_file"
            kind: "function"
            public: false
          - name: "format_e2e_discovery_strip_prefix_warn"
            kind: "function"
            public: true
          - name: "parse_cases"
            kind: "function"
            public: false
          - name: "extract_case_title"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/step_artifacts.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "STEP_ARTIFACTS_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "ArtifactLinkUnresolvable"
            kind: "enum"
            public: true
          - name: "ArtifactLinkResolution"
            kind: "enum"
            public: true
          - name: "ArtifactLinkRow"
            kind: "struct"
            public: true
          - name: "ScreenshotPreview"
            kind: "struct"
            public: true
          - name: "StepArtifactsPanel"
            kind: "struct"
            public: true
          - name: "from_context"
            kind: "function"
            public: true
          - name: "resolve"
            kind: "function"
            public: false
          - name: "path_unportable"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/screenshots.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "enum_model", "service_method"]
        symbols:
          - name: "SCREENSHOT_ARTIFACT_KIND"
            kind: "constant"
            public: true
          - name: "StepOutcome"
            kind: "enum"
            public: true
          - name: "is_failure"
            kind: "function"
            public: false
          - name: "ScreenshotMode"
            kind: "enum"
            public: true
          - name: "should_capture"
            kind: "function"
            public: true
          - name: "register_step_screenshot"
            kind: "function"
            public: true
          - name: "maybe_register_failed_step_screenshot"
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
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/open_controls.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "OPEN_RUN_CONTROL_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "PauseAnchor"
            kind: "enum"
            public: true
          - name: "PauseRequest"
            kind: "struct"
            public: true
          - name: "OpenRunMode"
            kind: "enum"
            public: true
          - name: "OpenRunCommand"
            kind: "enum"
            public: true
          - name: "OpenRunCommandError"
            kind: "enum"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "RunnerDecision"
            kind: "enum"
            public: true
          - name: "OpenRunControlState"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "apply"
            kind: "function"
            public: true
          - name: "mark_parked"
            kind: "function"
            public: true
          - name: "is_paused"
            kind: "function"
            public: true
          - name: "runner_decision_for"
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
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/playwright_shim.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "service_method"]
        symbols:
          - name: "MIGRATION_GUIDE_URL"
            kind: "constant"
            public: false
          - name: "SUPPRESS_ENV_VAR"
            kind: "constant"
            public: false
          - name: "safe_suppress_playwright_warning"
            kind: "function"
            public: true
          - name: "format_safe_suppress_warn"
            kind: "function"
            public: true
          - name: "PlaywrightArgs"
            kind: "struct"
            public: true
          - name: "run"
            kind: "function"
            public: true
          - name: "emit_deprecation_warning"
            kind: "function"
            public: true
          - name: "spawn_playwright"
            kind: "function"
            public: true
          - name: "safe_playwright_exit_code"
            kind: "function"
            public: true
          - name: "format_safe_playwright_exit_code_warn"
            kind: "function"
            public: true
          - name: "validate_no_native_flags"
            kind: "function"
            public: true
          - name: "imports_playwright_test"
            kind: "function"
            public: true
          - name: "tests"
            kind: "module"
            public: false
          - name: "gh3606_suppress_warn_tests"
            kind: "module"
            public: false
          - name: "gh3655_safe_playwright_exit_code_tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/step_panels.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "STEP_PANELS_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "ConsoleSeverity"
            kind: "enum"
            public: true
          - name: "classify"
            kind: "function"
            public: true
          - name: "ConsoleRow"
            kind: "struct"
            public: true
          - name: "from"
            kind: "function"
            public: false
          - name: "NetworkOutcome"
            kind: "enum"
            public: true
          - name: "classify"
            kind: "function"
            public: true
          - name: "NetworkRow"
            kind: "struct"
            public: true
          - name: "from"
            kind: "function"
            public: false
          - name: "PanelEmptyState"
            kind: "enum"
            public: true
          - name: "PanelProjection"
            kind: "enum"
            public: true
          - name: "is_empty"
            kind: "function"
            public: true
          - name: "row_count"
            kind: "function"
            public: true
          - name: "StepPanels"
            kind: "struct"
            public: true
          - name: "from_context"
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
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/trace.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "enum_model", "service_method"]
        symbols:
          - name: "TRACE_ARTIFACT_KIND"
            kind: "constant"
            public: true
          - name: "TraceMode"
            kind: "enum"
            public: true
          - name: "is_on"
            kind: "function"
            public: true
          - name: "register_case_trace"
            kind: "function"
            public: true
          - name: "maybe_register_case_trace"
            kind: "function"
            public: true
          - name: "content_type_for"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/open_state.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "OPEN_STATE_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "ShortcutAction"
            kind: "enum"
            public: true
          - name: "ShortcutBinding"
            kind: "struct"
            public: true
          - name: "default_shortcuts"
            kind: "function"
            public: true
          - name: "lookup_shortcut"
            kind: "function"
            public: true
          - name: "PanelLayout"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "OpenLocalState"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "load"
            kind: "function"
            public: true
          - name: "save"
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
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/open_replay.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "REPLAY_STEP_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "ReplayUnavailableReason"
            kind: "enum"
            public: true
          - name: "ui_label"
            kind: "function"
            public: true
          - name: "ReplayContext"
            kind: "struct"
            public: true
          - name: "ReplayStepAffordance"
            kind: "enum"
            public: true
          - name: "is_available"
            kind: "function"
            public: true
          - name: "unavailable_reason"
            kind: "function"
            public: true
          - name: "affordance_for"
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
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/video.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "enum_model", "service_method"]
        symbols:
          - name: "VIDEO_ARTIFACT_KIND"
            kind: "constant"
            public: true
          - name: "VideoMode"
            kind: "enum"
            public: true
          - name: "is_on"
            kind: "function"
            public: true
          - name: "VideoRegistrationError"
            kind: "enum"
            public: true
          - name: "fmt"
            kind: "function"
            public: false
          - name: "register_case_video"
            kind: "function"
            public: true
          - name: "maybe_register_case_video"
            kind: "function"
            public: true
          - name: "check_portable"
            kind: "function"
            public: false
          - name: "content_type_for"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method", "test_case"]
        symbols:
          - name: "actionability"
            kind: "module"
            public: true
          - name: "assertion_diff"
            kind: "module"
            public: true
          - name: "browser_session"
            kind: "module"
            public: true
          - name: "clock"
            kind: "module"
            public: true
          - name: "discovery"
            kind: "module"
            public: true
          - name: "dom_snapshot"
            kind: "module"
            public: true
          - name: "explorer"
            kind: "module"
            public: true
          - name: "lifecycle"
            kind: "module"
            public: true
          - name: "network"
            kind: "module"
            public: true
          - name: "open_controls"
            kind: "module"
            public: true
          - name: "open_replay"
            kind: "module"
            public: true
          - name: "open_state"
            kind: "module"
            public: true
          - name: "permissions"
            kind: "module"
            public: true
          - name: "playwright_shim"
            kind: "module"
            public: true
          - name: "retry"
            kind: "module"
            public: true
          - name: "screenshots"
            kind: "module"
            public: true
          - name: "selectors"
            kind: "module"
            public: true
          - name: "step_artifacts"
            kind: "module"
            public: true
          - name: "step_panels"
            kind: "module"
            public: true
          - name: "storage"
            kind: "module"
            public: true
          - name: "trace"
            kind: "module"
            public: true
          - name: "video"
            kind: "module"
            public: true
          - name: "EVIDENCE_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "CONTROL_PROTOCOL_VERSION"
            kind: "constant"
            public: true
          - name: "JET_BROWSER_DRIVER"
            kind: "constant"
            public: true
          - name: "JET_REVIEW_SHELL_DRIVER"
            kind: "constant"
            public: true
          - name: "E2E_EXIT_OK"
            kind: "constant"
            public: true
          - name: "E2E_EXIT_ASSERTION_FAILURE"
            kind: "constant"
            public: true
          - name: "E2E_EXIT_INVALID_CONFIG"
            kind: "constant"
            public: true
          - name: "E2E_EXIT_TIMEOUT"
            kind: "constant"
            public: true
          - name: "E2E_EXIT_INFRASTRUCTURE"
            kind: "constant"
            public: true
          - name: "E2eMode"
            kind: "enum"
            public: true
          - name: "E2eRunOptions"
            kind: "struct"
            public: true
          - name: "E2eOpenOptions"
            kind: "struct"
            public: true
          - name: "E2eRunResult"
            kind: "struct"
            public: true
          - name: "E2eEvidenceBundle"
            kind: "struct"
            public: true
          - name: "E2eSummary"
            kind: "struct"
            public: true
          - name: "E2eCaseEvidence"
            kind: "struct"
            public: true
          - name: "E2eProductStep"
            kind: "struct"
            public: true
          - name: "E2eAssertionDetail"
            kind: "struct"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "test"
          section_type: "tests"
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/storage.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "STORAGE_RESET_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "StorageSurface"
            kind: "enum"
            public: true
          - name: "StorageResetPolicy"
            kind: "struct"
            public: true
          - name: "default_between_cases"
            kind: "function"
            public: true
          - name: "none"
            kind: "function"
            public: true
          - name: "is_noop"
            kind: "function"
            public: true
          - name: "to_browser_script"
            kind: "function"
            public: true
          - name: "StorageResetEvent"
            kind: "struct"
            public: true
          - name: "StorageResetEvidence"
            kind: "struct"
            public: true
          - name: "from_policy"
            kind: "function"
            public: true
          - name: "record"
            kind: "function"
            public: true
          - name: "covers_policy"
            kind: "function"
            public: true
          - name: "FakeBrowserStorage"
            kind: "struct"
            public: true
          - name: "set_local"
            kind: "function"
            public: true
          - name: "set_session"
            kind: "function"
            public: true
          - name: "set_cookie"
            kind: "function"
            public: true
          - name: "apply_reset"
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
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/lifecycle.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "AUT_LIFECYCLE_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "AutPhase"
            kind: "enum"
            public: true
          - name: "AutPhaseOutcome"
            kind: "enum"
            public: true
          - name: "AutFailureKind"
            kind: "enum"
            public: true
          - name: "AutPhaseFailure"
            kind: "struct"
            public: true
          - name: "AutLifecycleEvent"
            kind: "struct"
            public: true
          - name: "succeeded"
            kind: "function"
            public: true
          - name: "skipped"
            kind: "function"
            public: true
          - name: "failed"
            kind: "function"
            public: true
          - name: "AutLifecycleRecord"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "push"
            kind: "function"
            public: true
          - name: "is_success"
            kind: "function"
            public: true
          - name: "first_failure"
            kind: "function"
            public: true
          - name: "exit_code"
            kind: "function"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "tests"
            kind: "module"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/permissions.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "PERMISSION_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "BrowserPermission"
            kind: "enum"
            public: true
          - name: "PermissionState"
            kind: "enum"
            public: true
          - name: "PermissionGrant"
            kind: "struct"
            public: true
          - name: "grant"
            kind: "function"
            public: true
          - name: "deny"
            kind: "function"
            public: true
          - name: "PermissionPolicy"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "with"
            kind: "function"
            public: true
          - name: "is_empty"
            kind: "function"
            public: true
          - name: "state_for"
            kind: "function"
            public: true
          - name: "PermissionQueryOutcome"
            kind: "enum"
            public: true
          - name: "PermissionEvidence"
            kind: "struct"
            public: true
          - name: "PermissionQueryRecord"
            kind: "struct"
            public: true
          - name: "from_policy"
            kind: "function"
            public: true
          - name: "observe"
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
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/network.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "NETWORK_OBSERVATION_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "NetworkRequest"
            kind: "struct"
            public: true
          - name: "NetworkResponse"
            kind: "struct"
            public: true
          - name: "NetworkFailureKind"
            kind: "enum"
            public: true
          - name: "NetworkFailure"
            kind: "struct"
            public: true
          - name: "NetworkObservation"
            kind: "struct"
            public: true
          - name: "completed"
            kind: "function"
            public: true
          - name: "failed"
            kind: "function"
            public: true
          - name: "is_completed"
            kind: "function"
            public: true
          - name: "StepNetworkRecord"
            kind: "struct"
            public: true
          - name: "new"
            kind: "function"
            public: true
          - name: "push"
            kind: "function"
            public: true
          - name: "is_empty"
            kind: "function"
            public: true
          - name: "len"
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
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/explorer.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ExplorerFilter"
            kind: "struct"
            public: true
          - name: "with_text"
            kind: "function"
            public: true
          - name: "include_tag"
            kind: "function"
            public: true
          - name: "exclude_tag"
            kind: "function"
            public: true
          - name: "tag_filter"
            kind: "function"
            public: false
          - name: "matches"
            kind: "function"
            public: false
          - name: "ExplorerView"
            kind: "struct"
            public: true
          - name: "from_manifest"
            kind: "function"
            public: true
          - name: "is_empty"
            kind: "function"
            public: true
          - name: "len"
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
          domain: "projects/jet/src/e2e"
      - path: "projects/jet/src/e2e/retry.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["config_surface", "data_model", "enum_model", "service_method"]
        symbols:
          - name: "RETRY_SCHEMA_VERSION"
            kind: "constant"
            public: true
          - name: "RetryBudget"
            kind: "struct"
            public: true
          - name: "default"
            kind: "function"
            public: false
          - name: "interval"
            kind: "function"
            public: true
          - name: "RetryOutcome"
            kind: "enum"
            public: true
          - name: "is_pass"
            kind: "function"
            public: true
          - name: "attempts"
            kind: "function"
            public: true
          - name: "RetryRecord"
            kind: "struct"
            public: true
          - name: "from_outcome"
            kind: "function"
            public: true
          - name: "AttemptResult"
            kind: "enum"
            public: true
          - name: "drive_retry"
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
          domain: "projects/jet/src/e2e"
```

## Tests
<!-- type: tests lang: yaml -->

```yaml
tests:
  coverage_kind: semantic
  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives
  evidence:
    source_tests: []
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/e2e/selectors.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/dom_snapshot.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/actionability.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/browser_session.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/assertion_diff.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/clock.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/discovery.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/step_artifacts.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/screenshots.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/open_controls.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/playwright_shim.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/step_panels.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/trace.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/open_state.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/open_replay.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/video.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/mod.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/storage.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/lifecycle.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/permissions.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/network.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/explorer.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: "projects/jet/src/e2e/retry.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this feature/domain semantic TD.
    impl_mode: hand-written
  - path: ".aw/tech-design/projects/jet/semantic/jet-e2e.md"
    action: verify
    section: unit-test
    impl_mode: hand-written
    description: |
      Traceability repair: hand-written TD section retained as the implementation edge during AW standardization.

```
