# Task: Restructure Input for Change 'fix-bare-pkg-alias'

## Step 1: Read Issues

Read these files using the Read tool:
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/fix-bare-pkg-alias/issues/issue_*.md` — each issue's title, body, and labels
- `/Users/chris.cheng/cclab/cclab-jet/cclab/changes/fix-bare-pkg-alias/user_input.md` — user's original description

Extract from each issue:
- **Title and scope**: what subsystem/module does it affect?
- **Labels**: especially `crate:*` labels
- **Dependencies**: does the issue reference other issues?

## Step 2: Group by Spec Boundary

A "group" = a set of issues that will produce ONE spec and ONE implementation unit.

### Same Group (merge together)
- Issues that modify the **same crate or module** (same `crate:*` labels)
- Issues with **data dependencies**
- Issues that are **subtasks** of each other
- Issues that **must be implemented atomically**

### Separate Groups (split apart)
- Issues that touch **different crates** with no shared interface
- Issues that can be **implemented and tested independently**

### Edge Cases
- **1 issue** → single group
- **All issues share the same crate** → default to single group unless clearly independent
- **2-3 issues, all closely related** → prefer single group

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
cclab sdd artifact restructure-input fix-bare-pkg-alias cclab/changes/fix-bare-pkg-alias/payloads/restructure-input.json
```