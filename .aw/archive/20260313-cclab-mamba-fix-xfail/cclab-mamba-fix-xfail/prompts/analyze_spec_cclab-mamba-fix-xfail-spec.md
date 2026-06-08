# Task: Analyze Spec 'cclab-mamba-fix-xfail-spec' for Change 'cclab-mamba-fix-xfail'

A skeleton has been generated at `specs/cclab-mamba-fix-xfail-spec.md`.

## Instructions

1. Read context:
   - `sdd_read_artifact(scope="read_path:changes/cclab-mamba-fix-xfail/proposal.md")` for spec_plan routing
   - `sdd_read_artifact(scope="read_path:changes/cclab-mamba-fix-xfail/reference_context.md")` if no proposal
2. Read the skeleton: `sdd_read_artifact(scope="read_path:changes/cclab-mamba-fix-xfail/specs/cclab-mamba-fix-xfail-spec.md")`
3. **IMPORTANT — `main_spec_ref`**: Check the spec frontmatter. If `main_spec_ref` is `~` (null),
   you MUST determine the target path in `cclab/specs/` where this spec will be merged.
   Format: `<scope>/<category>/<spec-id>.md` (e.g., `cclab-sdd/tools/new-feature.md`).
   Use `sdd_read_artifact(scope="read_path:specs")` to see existing spec groups.
   Pass it as the `main_spec_ref` parameter when calling the artifact tool.
   Also pass `merge_strategy` ("new", "append", or "replace") as a parameter.
4. Decide which sections to fill based on the nature of the change:
   - **overview** — always fill
   - **requirements** — always fill
   - **scenarios** — always fill
   - **diagrams** — fill if visual representation helps (API flows, data models, state machines)
   - **api_spec** — fill if change involves HTTP/RPC/event-driven/workflow APIs
   - **test_plan** — fill to define test cases (use Mermaid+ requirement diagram with BDD Given/When/Then)
   - **changes** — fill to list affected files
5. Call `sdd_artifact_create_change_spec` with section="overview" and the analysis result as content.
   But FIRST, update the skeleton frontmatter with your analysis:

## Expected Action

Call the artifact tool to write the **overview** section first. Pass the `fill_sections`
array as a parameter (e.g., `fill_sections=["overview", "requirements", "scenarios"]`).
Also pass `main_spec_ref` and `merge_strategy` as parameters if determined above.
The system persists these to frontmatter automatically.

Then call the artifact tool for each remaining section in sequence.

## MCP Tools

```
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chris.cheng/cclab/cclab-mamba", scope="read_path:changes/cclab-mamba-fix-xfail/proposal.md")
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chris.cheng/cclab/cclab-mamba", scope="read_path:changes/cclab-mamba-fix-xfail/specs/cclab-mamba-fix-xfail-spec.md")
mcp__cclab-mcp__sdd_read_artifact(project_path="/Users/chris.cheng/cclab/cclab-mamba", scope="read_path:specs")
mcp__cclab-mcp__sdd_artifact_create_change_spec(project_path="/Users/chris.cheng/cclab/cclab-mamba", change_id="cclab-mamba-fix-xfail", spec_id="cclab-mamba-fix-xfail-spec", section="overview", content="...", fill_sections=["overview", "requirements", "scenarios"], main_spec_ref="cclab-sdd/tools/example.md", merge_strategy="new")
```
