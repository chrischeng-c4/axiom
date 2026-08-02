"""Black-box contract for the greenfield project bootstrap chain (#1518, #2182, #3307).

Drives a real epic that carries a valid but never-before-registered
`app:workbench` tracker identity through the full, real bootstrap chain:
`aw goal wi` names the exact idempotent `aw conf init --project-label
app:workbench` producer and states the identity "must be registered before
atomization"; a real `aw conf init` run creates a genuine, discoverable
`apps/workbench/aw.toml`; a second real run is byte-identical and reports
already-registered; a real `aw meta init --project workbench` run produces
genuine META-doc producer output (created README/CONTRIBUTING/CAPABILITIES
files); and a final `aw goal wi` re-run on the same epic proves forward
progress -- it no longer asks for configuration, and instead lands on
exactly the same `aw wi plan` gate any already-registered project's fresh
epic would hit, proving the identity now routes through ordinary
atomization rather than looping back to bootstrap or silently vanishing.
"""

from __future__ import annotations

import shlex
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw

CASE_ID = "aw-core-client-new-project-epic-configuration-bootstrap"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "new-project-epic-configuration-bootstrap"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-new-project-epic-configuration-bootstrap"
)
ASSERTIONS = (
    "a real epic carrying a valid but unregistered `app:workbench` identity "
    "is dispatched by `aw goal wi` to exactly `aw conf init --project-label "
    "app:workbench`, with `completion.missing` literally stating "
    "'epic project `workbench` must be registered before atomization', and "
    "a real run of that exact command creates a genuine, discoverable "
    "`apps/workbench/aw.toml` (real `[project]`/`[[workspaces]]` tables) "
    "and prints a registration confirmation naming the same next command",
    "a second real `aw conf init --project-label app:workbench` run over "
    "the same target leaves the on-disk `aw.toml` byte-for-byte unchanged "
    "and reports the project already registered (idempotent configuration), "
    "a real `aw meta init --project workbench` run genuinely creates "
    "`apps/workbench/README.md`/`CONTRIBUTING.md`/`CAPABILITIES.md` on disk "
    "(real META-doc producers, not a no-op), and a final real `aw goal wi` "
    "re-run on the same epic no longer requests configuration and instead "
    "reaches exactly the same `aw wi plan --project workbench --json` gate "
    "any already-registered project's fresh epic would hit, proving genuine "
    "forward progress into atomization rather than a bootstrap loop",
)


def verify() -> list[str]:
    with project_fixture() as root:
        epic = run_aw(
            root, "wi", "create", "--title", "Workbench bootstrap epic", "--type", "epic", "--json"
        )
        epic_payload = final_json(epic)
        epic_slug = epic_payload["slug"]
        assert epic_payload.get("labels") in (None, []), epic_payload.get("labels")

        updated = final_json(
            run_aw(root, "wi", "update", epic_slug, "--add-label", "app:workbench", "--json")
        )
        assert "app:workbench" in updated.get("labels", []), updated

        before = final_json(run_aw(root, "goal", "wi", epic_slug))
        assert before["action"] != "blocked", before
        assert before["next"]["command"] == "aw conf init --project-label app:workbench", before["next"]
        assert (
            "epic project `workbench` must be registered before atomization"
            in before["completion"]["missing"]
        ), before["completion"]

        conf_args = shlex.split(before["next"]["command"])
        assert conf_args[0] == "aw", conf_args

        conf1 = run_aw(root, *conf_args[1:])
        assert "registered `workbench` at apps/workbench" in conf1.stdout, conf1.stdout
        assert "next: aw meta init --project workbench" in conf1.stdout, conf1.stdout
        toml_path = root / "apps" / "workbench" / "aw.toml"
        assert toml_path.exists(), "conf init did not create apps/workbench/aw.toml"
        toml_1 = toml_path.read_text(encoding="utf-8")
        assert 'name = "workbench"' in toml_1, toml_1
        assert 'label = "app:workbench"' in toml_1, toml_1
        assert "[[workspaces]]" in toml_1, toml_1

        conf2 = run_aw(root, *conf_args[1:])
        assert "already registered" in conf2.stdout, conf2.stdout
        toml_2 = toml_path.read_text(encoding="utf-8")
        assert toml_1 == toml_2, "idempotent re-run must not rewrite apps/workbench/aw.toml"

        readme_path = root / "apps" / "workbench" / "README.md"
        contributing_path = root / "apps" / "workbench" / "CONTRIBUTING.md"
        capabilities_path = root / "apps" / "workbench" / "CAPABILITIES.md"
        assert not readme_path.exists(), "README should not exist before meta init"

        meta = final_json(run_aw(root, "meta", "init", "--project", "workbench"))
        assert meta["schema_version"] == "aw.meta.v1", meta
        assert meta["status"] == "initialized", meta
        assert "apps/workbench" in meta["projects"], meta
        created_paths = {c["path"] for c in meta["changes"] if c["status"] == "created"}
        assert "apps/workbench/README.md" in created_paths, meta["changes"]
        assert "apps/workbench/CONTRIBUTING.md" in created_paths, meta["changes"]
        assert "apps/workbench/CAPABILITIES.md" in created_paths, meta["changes"]
        assert readme_path.exists(), "meta init did not actually create README.md on disk"
        assert contributing_path.exists(), "meta init did not actually create CONTRIBUTING.md on disk"
        assert capabilities_path.exists(), "meta init did not actually create CAPABILITIES.md on disk"

        after = final_json(run_aw(root, "goal", "wi", epic_slug))
        assert after["next"]["command"] != "aw conf init --project-label app:workbench", after["next"]
        assert after["next"]["command"] == "aw wi plan --project workbench --json", after["next"]
        assert "conf init" not in str(after["completion"]["missing"]), after["completion"]

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
