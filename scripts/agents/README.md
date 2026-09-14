# Fleet migration note

The renderer source is `scripts/agents/render_fleet.py`.

The new workflow and 226-role projection are PENDING ACTIVATION. They are
source candidates, not active runtime policy. Do not start a product writer
task or pilot as part of this migration.

The CLI canary allowed all six forbidden writes. A native read-only
`lumen-qa` canary also wrote its first `/private/tmp` sentinel. The desktop
admission test ignored the installed hook and launched `lumen-dev`; it did no
tool or write action. The global configuration was restored. Only the
`lumen-qa` Luna/max declaration was measured. These results do not prove
permission isolation or hook enforcement for Codex or Claude.

Rollback is complete and verified. It restored 291 original role, config, and
hook files. It moved 168 new role files recoverably. `.codex`,
`.claude/agents`, and `.claude/hooks` are clean. The global configuration
matches original hash `61a65c...`.

The candidate archive is
`/private/tmp/axiom-rollback-reviewed/current-candidate.tar.gz`. Its
`moved-new` directory and `rollback-journal.json` are in the same directory.
The corrected draft hook tests remain in
`/private/tmp/axiom-fleet-phase2-protected`; staged and install-shape tests
there pass 10 and 16 cases.

Source templates, skills, and routing remain, but rollout is incomplete. The
current `render_fleet --check` deliberately reports 444 divergences and exits
1 after rollback. The product-skill check passes. Do not run `--write` now.
Before resuming, recover and review the archived singleton source candidates:
`aw-dev`, `cto`, `project-manager`, `tech-design`, and `integration-qa`.

This note records migration state only. It is not delivery evidence or an
approval to write.
