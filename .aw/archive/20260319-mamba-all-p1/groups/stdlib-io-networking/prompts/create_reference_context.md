# Task: Gather Reference Context for Group 'stdlib-io-networking' (Change 'mamba-all-p1')

Issues: #661_add-native-stdlib-ssl-tls-ssl-wrapper-for-socket-o, #658_add-native-stdlib-selectors-high-level-i-o-multipl, #662_add-native-stdlib-urllib-url-handling-modules, #664_add-native-stdlib-multiprocessing-process-based-pa, #665_add-native-stdlib-concurrent-futures-async-executi, #663_add-native-stdlib-email-email-handling-package

## Instructions

Specs are the **single source of truth**.

1. **Understand scope**: Read group pre-clarifications to identify which crates/areas are in scope:
   `/Users/chris.cheng/cclab/cclab-mamba/cclab/changes/mamba-all-p1/groups/stdlib-io-networking/pre_clarifications.md`
2. **Identify candidate specs**: Read relevant specs (see below)
3. **Evaluate relevance**: For each candidate spec, reason about its relevance:
   - high = directly implements the group's requirements
   - medium = related/supporting
   - low = background context only
4. **Self-verify before submitting**: Check — does every crate/area from pre-clarifications have at least one spec covering it? If not, search for missing specs.
5. Run `cclab sdd artifact create-reference-context` with the structured `specs` array

## In-Scope Specs

### cclab-mamba
- `read_path:specs/cclab-mamba/README.md`
- `read_path:specs/cclab-mamba/all-mamba-p0.md`
- `read_path:specs/cclab-mamba/pattern-matching.md`


Read these specs using the Read tool (file paths under `/Users/chris.cheng/cclab/cclab-mamba/cclab/specs/`).
Do NOT explore specs outside the scope above.

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
cclab sdd artifact create-reference-context mamba-all-p1 cclab/changes/mamba-all-p1/payloads/create-reference-context.json
```