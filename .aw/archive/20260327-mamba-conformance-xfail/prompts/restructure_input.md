# Task: Restructure Input for Change 'mamba-conformance-xfail'

## Step 1: Read Input

Read:
- `/Users/chris.cheng/cclab/cclab-sdd-mamba-conformance/cclab/changes/mamba-conformance-xfail/user_input.md` — user's description

## Step 2: Determine Group

Since there are no issues, create a single group with:
- `id`: derived from the change description (kebab-case)
- `issues`: empty array `[]`

## Step 3: Consolidate Requirements

For each group, write a consolidated requirements summary:
- What needs to be built/changed
- Key constraints and acceptance criteria
- Integration points with existing code

## Step 4: Generate Questions

For each group, generate clarification questions:
- Ambiguities in scope or requirements
- Missing technical details (which modules, what APIs, etc.)
- Implementation choices that need user input

## Step 5: Self-Review Checklist

Before calling the artifact tool, verify:
- [ ] Every issue appears in exactly one group (if issues exist)
- [ ] Each group has a clear, consolidated requirements summary
- [ ] Questions are specific and actionable (not generic)
- [ ] Group IDs are kebab-case and descriptive

## Step 6: Write Result

Run `cclab sdd artifact restructure-input` with the restructured result.

## CLI Commands

```
# Write artifact (write payload JSON first, then run)
cclab sdd artifact restructure-input mamba-conformance-xfail cclab/changes/mamba-conformance-xfail/payloads/restructure-input.json
```