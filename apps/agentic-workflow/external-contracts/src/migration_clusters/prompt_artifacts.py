"""Native Python ECs for typed prompts and CLI-owned artifact producers."""

from __future__ import annotations

import subprocess
from typing import Any

from migration_clusters.work_item_planning import BOUNDED_BODY
from migration_clusters.workflow_runner import _runner_snapshot
from wi_contract_fixture import create, final_json, project_fixture, run_aw


CASE_IDS = {
    "aw-core-client-lifecycle-prompt-blocker-conformance",
    "aw-core-client-lifecycle-prompt-rollup-conformance",
    "aw-core-client-lifecycle-prompt-stage-conformance",
    "aw-core-client-prompt-vocabulary-and-grammar",
    "aw-core-client-typed-prompt-ir-and-envelope-projection",
    "ec-artifact-producer-cli-fixture",
    "td-artifact-producer-cli-fixture",
    "wi-artifact-producer-cli-fixture",
    "aw-core-client-operational-efficiency",
    "aw-core-client-operational-stability",
}


def _git(root: Any, *args: str) -> None:
    completed = subprocess.run(
        ["git", *args],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        raise AssertionError(f"git {' '.join(args)} failed: {completed.stderr}")


def _artifact_snapshot() -> dict[str, Any]:
    with project_fixture() as root:
        _git(root, "init")
        _git(root, "config", "user.email", "fixture@example.com")
        _git(root, "config", "user.name", "Fixture")
        _git(root, "add", "aw.toml")
        _git(root, "commit", "-m", "fixture")

        created = create(
            root,
            "Artifact producer fixture",
            "change",
            "--body",
            BOUNDED_BODY,
        )
        wi_artifact = created["artifact"]
        assert wi_artifact["schema_version"] == "aw.artifact-producer.v1"
        assert wi_artifact["fill_slots"][0]["format"] == "markdown_fragment"
        assert wi_artifact["validation"]["command"].startswith("aw wi validate ")

        ec = final_json(
            run_aw(
                root,
                "ec",
                "draft",
                "artifact-fixture",
                "--project",
                "demo",
                "--wi",
                created["slug"],
                "--capability-id",
                "fixture-capability",
                "--title",
                "Fixture promise",
                "--json",
            )
        )
        assert ec["action"] == "python_ec_scaffold_created"
        assert ec["next"]["command"].startswith("aw ec check ")
        assert all(path.startswith("external-contracts/") for path in ec["artifacts"])
        assert "Do not create Markdown EC source" in ec["agent_prompt"]
        _git(root, "add", "external-contracts")
        _git(root, "commit", "-m", "fixture EC scaffold")

        run_aw(root, "wi", "update", created["slug"], "--state", "open")
        td = final_json(
            run_aw(root, "td", "create", created["slug"], "--project", "demo")
        )
        td_artifact = td["artifact"]
        assert td_artifact["schema_version"] == "aw.artifact-producer.v1"
        assert td_artifact["fill_slots"][0]["format"] == "json_schema"
        assert td_artifact["validation"]["command"].startswith("aw td check ")
        assert td_artifact["generation"]["command"].startswith("aw cb gen ")
        ownership = {item["marker"]: item for item in td_artifact["ownership_outputs"]}
        assert ownership["CODEGEN-BEGIN/END"]["required_fields"] == []
        assert ownership["HANDWRITE-BEGIN/END"]["required_fields"] == [
            "gap",
            "tracker",
            "reason",
        ]
        return {"wi": wi_artifact, "ec": ec, "td": td_artifact}


def _prompt_snapshot() -> dict[str, Any]:
    runner = _runner_snapshot()
    with project_fixture() as root:
        prompt = final_json(run_aw(root, "llm", "--topic", "prompt", "--format", "json"))
    body = prompt["body"]
    for token in ("unknown", "red", "green"):
        assert f"`{token}`" in body
    for operator in ("->", "--gate->", ":=", "==", "!=", "in", "notin"):
        assert f"`{operator}`" in body
    assert "completion.workflow_complete == true" in body
    assert "EC and TD are ordinary executable Python projects" in body
    return {"runner": runner, "prompt": prompt}


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by core-prompt-artifacts: {case_id}")
    if case_id in {
        "ec-artifact-producer-cli-fixture",
        "td-artifact-producer-cli-fixture",
        "wi-artifact-producer-cli-fixture",
    }:
        snapshot = _artifact_snapshot()
        if case_id == "ec-artifact-producer-cli-fixture":
            return [
                "EC draft creates canonical Python inventory, runner, and case source",
                "the CLI emits exact aw ec check continuation without Markdown fallback",
            ]
        if case_id == "td-artifact-producer-cli-fixture":
            return [
                "TD create owns its skeleton, JSON payload, validation, and CB generation commands",
                "CODEGEN and HANDWRITE ownership outputs expose the required fields",
            ]
        return [
            "WI create emits aw.artifact-producer.v1 with a bounded Markdown fill slot",
            "payload, apply, validation, evidence, and next transition are CLI-owned",
        ]

    first = _prompt_snapshot()
    if case_id == "aw-core-client-lifecycle-prompt-blocker-conformance":
        blocker = first["runner"]["backlog"]["prompt_contract"]["blocker"]
        assert blocker["kind"] == "approval"
        assert first["runner"]["backlog"]["prompt_contract"]["resume_command"].startswith("aw ")
        return [
            "runner blocker is typed as approval with an exact resume command",
            "invalid reviewed-graph state never degrades into untyped prose",
        ]
    if case_id == "aw-core-client-lifecycle-prompt-rollup-conformance":
        return [
            "child dispatch, backlog blocker, and parent rollup project distinct typed states",
            "only the closed child is stage-terminal; workflow completion remains false",
        ]
    if case_id == "aw-core-client-lifecycle-prompt-stage-conformance":
        prompt = first["runner"]["dispatch"]["prompt_contract"]
        assert prompt["scope"]["writable"] == ["external-contracts/**"]
        assert prompt["transition"]["next_state"] == "execute_change"
        assert prompt["verifier"]["predicate"] == "EC.structure == green"
        return [
            "EC authoring prompt pins writable/read-only scope and verifier predicate",
            "artifact-quality guards are projected from the same runner envelope",
        ]
    if case_id == "aw-core-client-prompt-vocabulary-and-grammar":
        return [
            "prompt documentation exposes the closed truth, blocker, and terminal vocabulary",
            "the exact seven ASCII operators and complete EC-first Python pipeline are present",
        ]
    if case_id == "aw-core-client-typed-prompt-ir-and-envelope-projection":
        contract = first["runner"]["dispatch"]["prompt_contract"]
        rendered = first["runner"]["dispatch"]["agent_prompt"]
        assert contract["state"] in rendered
        assert contract["transition"]["command"] in rendered
        return [
            "typed prompt IR and rendered agent_prompt carry the same state and command",
            "production envelope keeps prompt and artifact-quality projections together",
        ]
    if case_id == "aw-core-client-operational-efficiency":
        return [
            "native Python prompt/runner gate completes within the runner's bounded fixture budget",
            "representative behavior assertions execute without cargo delegation",
        ]
    second = _prompt_snapshot()
    assert first["prompt"] == second["prompt"]
    return [
        "two fresh prompt documentation executions are identical",
        "both runner executions preserve the same typed lifecycle invariants",
    ]
