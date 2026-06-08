# Tasks

## 1. Core: Phase State Machine

- [ ] 1.1 Update StatePhase enum (add Rejected, remove Testing)
  - File: `src/models/frontmatter.rs` (MODIFY)
  - File: `src/models/change.rs` (MODIFY)
  - File: `genesis/schemas/state.schema.json` (MODIFY)
  - Spec: `specs/workflows.md#r1`
  - Do:
    - Add `Rejected` variant to `StatePhase` enum
    - Remove deprecated `Testing` variant (use `Implementing` instead)
    - Update schema to match
  - Depends: none

- [ ] 1.2 Update challenge command to set phase based on verdict
  - File: `src/cli/challenge_proposal.rs` (MODIFY)
  - File: `src/cli/validate_challenge.rs` (MODIFY)
  - Spec: `specs/workflows.md#r2`
  - Do: After challenge completes, update STATE.yaml phase:
    - APPROVED → `phase: challenged`
    - NEEDS_REVISION → `phase: proposed` (stays, triggers auto-reproposal)
    - REJECTED → `phase: rejected`
  - Depends: 1.1

- [ ] 1.3 Update archive command to set phase to archived
  - File: `src/cli/archive.rs` (MODIFY)
  - Spec: `specs/workflows.md#r5`
  - Do: Set `phase: archived` after successful archive completion
  - Depends: 1.1

- [ ] 1.4 Update status display for rejected phase
  - File: `src/cli/status.rs` (MODIFY)
  - Do: Add icon and message for `rejected` phase (e.g., ⛔ Rejected)
  - Depends: 1.1

## 2. Skill Layer

- [ ] 2.1 Create `genesis:plan` skill template
  - File: `templates/skills/genesis-plan/SKILL.md` (CREATE)
  - Spec: `specs/workflows.md#r3`
  - Do: Define skill with phase-only logic:
    - No STATE.yaml → require description, run `genesis proposal <change-id> "<description>"`
    - `proposed` → run `genesis proposal` (description optional, reuses existing)
    - `challenged` → "✅ Planning complete, run /genesis:impl"
    - `rejected` → "⛔ Rejected, review CHALLENGE.md"
    - Other → "ℹ️ Beyond planning phase"
  - Input: `/genesis:plan <change-id> ["<description>"]`
  - Note: `genesis proposal` internally manages challenge + auto-reproposal loop
  - Depends: 1.2

- [ ] 2.2 Create `genesis:impl` skill template
  - File: `templates/skills/genesis-impl/SKILL.md` (CREATE)
  - Spec: `specs/workflows.md#r4`
  - Do: Define skill with phase-only logic:
    - `challenged` or `implementing` → `genesis implement`
    - Other → "❌ ChangeNotReady"
  - Depends: 1.2

- [ ] 2.3 Update `genesis:archive` skill template
  - File: `templates/skills/genesis-archive/SKILL.md` (MODIFY)
  - Spec: `specs/workflows.md#r5`
  - Do: Ensure phase-only logic:
    - `complete` → `genesis archive`
    - Other → "❌ ChangeNotComplete"
  - Depends: 1.3

- [ ] 2.4 Deprecate granular skills
  - Files:
    - `templates/skills/genesis-proposal/SKILL.md` (MODIFY)
    - `templates/skills/genesis-challenge/SKILL.md` (MODIFY)
    - `templates/skills/genesis-reproposal/SKILL.md` (MODIFY)
    - `templates/skills/genesis-implement/SKILL.md` (MODIFY)
    - `templates/skills/genesis-review/SKILL.md` (MODIFY)
    - `templates/skills/genesis-resolve-reviews/SKILL.md` (MODIFY)
  - Do: Add deprecation notice: "⚠️ DEPRECATED: Use /genesis:plan, /genesis:impl, or /genesis:archive instead"
  - Note: Use `genesis-resolve-reviews` (not `genesis-fix`) as canonical name
  - Depends: 2.1, 2.2, 2.3

## 3. Init & Sync

- [ ] 3.1 Update `init` command to include new skills
  - File: `src/cli/init.rs` (MODIFY)
  - Do: Add `genesis-plan` and `genesis-impl` to skill installation list
  - Depends: 2.1, 2.2

- [ ] 3.2 Sync skills to `.claude/skills/`
  - File: `.claude/skills/genesis-plan/SKILL.md` (CREATE)
  - File: `.claude/skills/genesis-impl/SKILL.md` (CREATE)
  - File: `.claude/skills/genesis-archive/SKILL.md` (UPDATE)
  - Do: Copy new skill templates to active skill directories
  - Note: Can also run `genesis init --force` to sync all skills
  - Depends: 2.1, 2.2, 2.3

- [ ] 3.3 Update CLAUDE.md template
  - File: `templates/CLAUDE.md` (MODIFY)
  - Do: Update skill table:
    ```
    | /genesis:plan    | Planning workflow (proposal → challenge) |
    | /genesis:impl    | Implementation workflow                  |
    | /genesis:archive | Archive completed change                 |
    ```
  - Depends: 2.1, 2.2, 2.3

## 4. Testing

- [ ] 4.1 Test phase transitions
  - File: `src/state/manager.rs` (MODIFY - add tests)
  - Verify: `specs/workflows.md#acceptance-criteria`
  - Do: Test that:
    - Challenge verdict correctly sets phase (APPROVED→challenged, REJECTED→rejected)
    - Archive sets phase to archived
    - Status displays rejected phase correctly
  - Depends: 1.1, 1.2, 1.3, 1.4
