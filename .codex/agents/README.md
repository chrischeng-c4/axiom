# Codex agent fleet — projection of `.claude/agents/`

This directory holds the Codex-runtime projection of the Claude agent fleet:
one `<name>.toml` per `.claude/agents/<name>.md`, 91 in total. The Claude
markdown definition is the source of truth; each TOML carries the same
`name`, `description`, pinned reasoning effort, and the full markdown body as
`developer_instructions`. The generator is `scripts/agents/render_fleet.py`:
`--write` re-renders every projection after a `.claude/agents/` change and
removes any `*.toml` with no markdown twin, so a removed Claude agent cannot
survive here as a stale role; `--check` (run by
`.codex/hooks/test_require_spawn_agent_effort.py`) refuses a hand-edited
projection. The per-project markdown itself is rendered from
`scripts/agents/templates/<tier>/<role>.md` — edit the template, never one
project's copy.

The fleet is two agents per project (22 apps and 22 libs) plus `aw-dev` and
two operators:

| Role | Effort | Owns |
|---|---|---|
| `<p>-e2e-dev` (44) | `max` | the e2e contract — behavior, performance, and security facets, written to fail first; never writes `src/` |
| `<p>-dev` (44) | `medium` | source plus colocated unit tests, verified by running them; never writes `e2e/` |
| `aw-dev` | `medium` | bounded changes to the `apps/aw` Python CLI, pytest via uv |
| `agy-operator` | `low` | one frozen AGY dispatch round |
| `gke-operator` | `medium` | babysitting the paid GKE acceptance harness |

Unlike the retired single-role fleet, **effort is pinned per role**, exactly
as the Claude frontmatter pins it. Dispatch must pass the same value as
`reasoning_effort`; `.codex/hooks/require_spawn_agent_effort.py` refuses a
spawn whose effort does not match the named `agent_type`'s
`model_reasoning_effort`, mirroring `.claude/hooks/require_agent_effort.py`.
A hard case may still be raised at dispatch time — by editing nothing and
overriding `model` in the spawn call — but phase ownership does not move
with the model.

Every role fixes `model = "gpt-5.6-terra"`. The Claude per-project fleet is
sonnet throughout; its tiers differ by pinned effort (`max` vs `medium`),
and that effort split is what this projection carries.
Two Claude frontmatter fields have no Codex TOML equivalent and are not
projected: `tools` (Codex has no per-tool allowlist; the `## Never` sections
carry the same boundaries as prose) and `skills` (the bodies name the
`/aw-e2e-for` / `/aw-impl-for` workflows directly).

See `AGENTS.md`, "Use Codex subagents; effort is pinned per role", for the
division of labor this fleet implements.
