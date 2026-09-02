# Codex agent fleet — projection of `.claude/agents/`

This directory holds the Codex-runtime projection of the Claude agent fleet:
one `<name>.toml` per `.claude/agents/<name>.md`, 91 in total. The Claude
markdown definition is the source of truth; each TOML carries the same
`name`, `description`, pinned reasoning effort, and the full markdown body as
`developer_instructions`. Regenerate after any `.claude/agents/` change —
the generator deletes every `*.toml` first, so a removed Claude agent cannot
survive here as a stale role.

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

Every role fixes `model = "gpt-5.6-terra"`. The Claude fleet's opus/sonnet
tier split maps onto the pinned effort split (`max` vs `medium`) instead.
Two Claude frontmatter fields have no Codex TOML equivalent and are not
projected: `tools` (Codex has no per-tool allowlist; the `## Never` sections
carry the same boundaries as prose) and `skills` (the bodies name the
`/aw-e2e-for` / `/aw-impl-for` workflows directly).

See `.claude/rules/operations/team-division.md` for the division of labor
this fleet implements.
