# Code Review (Iteration 0)

## Test Results
- **Status**: not_run

## Security
- **Status**: not_run (pip-audit/semgrep unavailable)

## Issues

### MEDIUM
1. **SpecType diagram requirements enforced as AND instead of OR**
   - SemanticValidator enforces every diagram in `required_diagrams_as_strings`, which makes `data-model`, `algorithm`, and `workflow` require both diagram types instead of the intended OR semantics (ERD or class, flowchart or state, etc.). Valid specs will fail validation.
   - Location: `crates/cclab-genesis/src/validator/semantic.rs:64`
   - Recommendation: Mirror the OR logic used in `spec_service::validate_spec_type_requirements` or encode alternatives in `spec_rules` so validation matches the spec requirements.

2. **Diagram type detection is too broad and bypasses enforcement**
   - `contains_diagram` treats any ```mermaid``` block as satisfying the required diagram type, so a non-sequence diagram can pass the sequence requirement. This makes spec_type enforcement ineffective and can yield false positives.
   - Location: `crates/cclab-genesis/src/validator/semantic.rs:135`
   - Recommendation: Detect the actual diagram type from the Mermaid code (e.g., `sequenceDiagram`, `flowchart`, etc.) or parse the structured `design_elements.diagrams` list; avoid matching any Mermaid block.

3. **Invalid spec_type not reported as error**
   - Invalid `spec_type` values are silently ignored in SemanticValidator. Acceptance criteria require invalid spec_type to fail validation, but no error is emitted when `SpecType::from_str` fails.
   - Location: `crates/cclab-genesis/src/validator/semantic.rs:61`
   - Recommendation: Emit a `ValidationError` when `SpecType::from_str` fails so unsupported `spec_type` values are surfaced as errors.

4. **validate_spec_completeness not aligned with SemanticValidator**
   - `validate_spec_completeness` still uses its own parsing logic and does not delegate to `SemanticValidator`, so it can diverge from the centralized rules (violates R5/acceptance criteria for consistency).
   - Location: `crates/cclab-genesis/src/mcp/tools/validate_spec.rs:1`
   - Recommendation: Refactor this tool to call the enhanced `SemanticValidator` and surface its results, or share a single validation utility so outputs remain consistent.

5. **Spec-type guidance missing from get_task tool**
   - `get_task` has no spec_type-based guidance and does not use centralized rules, so R7 guidance enhancements are not implemented.
   - Location: `crates/cclab-genesis/src/mcp/tools/task.rs:1`
   - Recommendation: Inject spec_type guidance for create/revise spec tasks using `spec_rules` and expose required diagrams/API specs in the rendered template.

6. **Unscoped documentation deletion**
   - `cclab/project.md` was deleted but the proposal/specs do not mention removing project documentation; this looks like scope creep.
   - Location: `cclab/project.md:1`
   - Recommendation: Confirm whether this deletion is intended for this change; otherwise restore it to keep the change focused.

7. **Unused import in benchmark setup**
   - `asyncio` is imported but unused, which was flagged in the lint output and adds noise to the change.
   - Location: `python/tests/api/benchmarks/benchmark_setup.py:9`
   - Recommendation: Remove the unused `asyncio` import or use it if intended.

## Verdict
NEEDS_CHANGES

**Next Steps**: Address the spec_type validation logic in SemanticValidator and align validate_spec_completeness/get_task with centralized rules, then rerun relevant tests.
