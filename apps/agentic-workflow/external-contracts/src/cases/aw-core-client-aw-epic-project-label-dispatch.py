"""Black-box contract for epic project-label resolution and dispatch (#1518, #2182, #3307).

Drives a three-project fixture (`app:demo`, `app:otherapp`, `lib:libthing`
-- covering all three supported identity prefixes) through the real `aw
goal wi`/`aw wi plan` pipeline and proves two real behaviors: an epic with
no recognized `project:`/`app:`/`lib:` identity label blocks safely with a
runnable remediation and no atomize/dispatch command (never a placeholder),
and three concurrently open, fully-planned epics -- one per label prefix --
each dispatch strictly to their own project's ready child change and never
to a sibling project's, proving label resolution is genuinely per-epic and
per-project rather than coincidentally right for a single fixture.
"""

from __future__ import annotations

import shlex
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import final_json, run_aw

CASE_ID = "aw-core-client-aw-epic-project-label-dispatch"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "aw-epic-project-label-dispatch"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-aw-epic-project-label-dispatch"
)
ASSERTIONS = (
    "a real, freshly created epic carrying no `project:`/`app:`/`lib:` "
    "identity label is dispatched by `aw goal wi` as a blocked HITL "
    "envelope whose `prompt_contract.blocker.kind == \"decision\"`, whose "
    "reason names all three supported prefixes and a concrete `aw wi show "
    "<id>` remediation, and whose `next.command` is that remediation -- "
    "never an atomize or dispatch command -- proving an unresolved epic "
    "fails closed instead of emitting a placeholder handoff",
    "three real, concurrently open epics registered under three distinct "
    "supported label prefixes (`app:demo`, `app:otherapp`, `lib:libthing`), "
    "each carried through the real `aw wi plan` pipeline to a fully "
    "verified project graph, each dispatch via `aw goal wi` to exactly "
    "`aw goal wi <own-project's-ready-child>` -- strictly its own child's "
    "id, confirmed never equal to either sibling's child id -- proving "
    "project-label resolution and epic-to-child routing is genuinely "
    "per-epic and per-project rather than a single-fixture coincidence",
)

_FIXTURE_AW_TOML = """
[agentic_workflow.workspace]
mode = "in_place"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "demo"
label = "app:demo"
path = "."
tech_design_path = "tech-design"

[[projects.workspaces]]
name = "demo"
paths = ["**"]
target = "rust"

[[projects]]
name = "otherapp"
label = "app:otherapp"
path = "apps/otherapp"
tech_design_path = "tech-design"

[[projects.workspaces]]
name = "otherapp"
paths = ["apps/otherapp/**"]
target = "rust"

[[projects]]
name = "libthing"
label = "lib:libthing"
path = "libs/libthing"
tech_design_path = "tech-design"

[[projects.workspaces]]
name = "libthing"
paths = ["libs/libthing/**"]
target = "rust"
""".lstrip()


def _create_epic(root: Path, title: str, project: str | None = None) -> dict:
    args = ["wi", "create", "--title", title, "--type", "epic", "--json"]
    if project:
        args += ["--project", project]
    return final_json(run_aw(root, *args))


def _create_change(root: Path, title: str, project: str, epic_slug: str) -> dict:
    return final_json(
        run_aw(
            root,
            "wi",
            "create",
            "--title",
            title,
            "--type",
            "change",
            "--project",
            project,
            "--epic",
            epic_slug,
            "--json",
        )
    )


def _drive_plan(root: Path, project: str) -> None:
    args = ["wi", "plan", "--project", project, "--json"]
    for _ in range(12):
        result = run_aw(root, *args, expect_success=None)
        assert result.returncode == 0, (args, result.stdout, result.stderr)
        env = final_json(result)
        if env.get("completion", {}).get("workflow_complete"):
            return
        assert not env.get("requires_hitl") and env.get("action") != "blocked", env
        parts = shlex.split(env["next"]["command"])
        assert parts[0] == "aw", parts
        args = parts[1:]
    raise AssertionError(f"plan pipeline for {project} did not converge")


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-ec-epic-label-dispatch-") as raw_root:
        root = Path(raw_root)
        (root / "aw.toml").write_text(_FIXTURE_AW_TOML, encoding="utf-8")
        (root / "apps" / "otherapp").mkdir(parents=True)
        (root / "libs" / "libthing").mkdir(parents=True)

        bare = _create_epic(root, "Bare unlabeled epic")
        bare_slug = bare["slug"]
        assert bare.get("labels") in (None, []), bare.get("labels")
        blocked_env = final_json(run_aw(root, "goal", "wi", bare_slug))
        assert blocked_env["action"] == "blocked", blocked_env
        blocker = blocked_env["prompt_contract"]["blocker"]
        assert blocker["kind"] == "decision", blocker
        for prefix in ("project:", "app:", "lib:"):
            assert prefix in blocker["reason"], blocker["reason"]
        assert blocked_env["next"]["command"] == f"aw wi show {bare_slug}", blocked_env["next"]
        assert "atomize" not in blocked_env["next"]["command"], blocked_env["next"]
        assert "dispatch" not in blocked_env["action"], blocked_env

        child_slugs: dict[str, str] = {}
        for project in ("demo", "otherapp", "libthing"):
            epic = _create_epic(root, f"{project} routing epic", project=project)
            epic_slug = epic["slug"]
            change = _create_change(root, f"{project} routing change", project, epic_slug)
            child_slugs[project] = change["slug"]
            _drive_plan(root, project)
            env = final_json(run_aw(root, "goal", "wi", epic_slug))
            assert env["action"] != "blocked", (project, env)
            expected = f"aw goal wi {child_slugs[project]}"
            assert env["next"]["command"] == expected, (project, env["next"])

        assert len(set(child_slugs.values())) == 3, child_slugs
        for project, slug in child_slugs.items():
            others = {p: s for p, s in child_slugs.items() if p != project}
            assert slug not in others.values(), (project, slug, others)

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
