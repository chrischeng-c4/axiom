"""Contract tests for Cue app goals and success metrics."""

from __future__ import annotations

import asyncio

from test_workstream_api import _load_main


def test_app_spec_goal_gates_production_but_not_sandbox() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/admin/app-spec/validate")]
    spec = {
        "schema_version": "cue.app-spec.v0",
        "app_id": "goal-missing",
        "name": "Goal Missing",
        "owner_team": "Operations",
        "owner_user": "ops@example.com",
        "lifecycle_status": "draft",
    }

    sandbox = asyncio.run(handler({"phase": "sandbox", "spec": spec}))
    production = asyncio.run(handler({"phase": "production", "spec": spec}))

    assert sandbox["data"]["valid"] is True
    assert {warning["code"] for warning in sandbox["data"]["warnings"]} >= {
        "goal_statement_missing",
        "goal_success_metric_missing",
    }
    assert production["data"]["valid"] is False
    assert production["data"]["deployment_gate"] == "blocked"
    assert {error["code"] for error in production["data"]["errors"]} >= {
        "goal_statement_missing",
        "goal_success_metric_missing",
    }


def test_nested_app_spec_goal_is_valid_for_production_and_preview() -> None:
    main = _load_main()
    validate = main.app._handlers[("POST", "/api/admin/app-spec/validate")]
    preview = main.app._handlers[("POST", "/api/admin/app-spec/preview")]
    spec = {
        "schema_version": "cue.app-spec.v0",
        "app": {
            "id": "goal-ready",
            "name": "Goal Ready",
            "owner_team": "Operations",
            "owner_user": "ops@example.com",
            "lifecycle_status": "draft",
            "goal": {
                "statement": "Reduce request cycle time.",
                "problem": "Manual handoffs hide aging work.",
                "success_metrics": [{"name": "cycle_time", "baseline": "5 days", "target": "2 days", "window": "90 days"}],
                "review_policy": {"cadence": "monthly", "owner_required": True},
            },
        },
    }

    result = asyncio.run(validate({"phase": "production", "spec": spec}))
    rendered = asyncio.run(preview({"spec": spec}))

    assert result["data"]["valid"] is True
    assert result["data"]["deployment_gate"] == "ready"
    assert rendered["data"]["goal_preview"]["statement"] == "Reduce request cycle time."
    assert rendered["data"]["goal_preview"]["success_metrics"][0]["name"] == "cycle_time"


def test_registry_goal_review_and_goal_updates_are_auditable() -> None:
    main = _load_main()
    catalog_handler = main.app._handlers[("POST", "/api/admin/app-registry/catalog")]
    review_handler = main.app._handlers[("GET", "/api/admin/projects/{project_id}/goal-review")]
    update_handler = main.app._handlers[("POST", "/api/admin/projects/{project_id}/goal")]
    evidence_handler = main.app._handlers[("GET", "/api/admin/workitems/{workitem_id}/evidence")]

    catalog = asyncio.run(catalog_handler({"query": "request"}))
    review = asyncio.run(review_handler("team-request-tracker"))
    updated = asyncio.run(
        update_handler(
            "team-request-tracker",
            {
                "actor": "ops-lead@example.com",
                "goal": {
                    "statement": "Cut internal request aging.",
                    "problem": "Owners need a visible queue.",
                    "target_users": ["requester", "operations-owner"],
                    "success_metrics": [{"name": "cycle_time", "baseline": "5 days", "target": "2 days", "window": "90 days"}],
                    "review_policy": {"cadence": "monthly", "owner_required": True},
                },
            },
        )
    )
    evidence = asyncio.run(evidence_handler("request-tracker-prd"))

    assert catalog["data"][0]["goal_status"]["status"] == "on_track"
    assert review["data"]["lifecycle_recommendation"] == "continue"
    assert review["data"]["retirement_inputs"]["active_users"] > 0
    assert updated["data"]["goal"]["version"] == 2
    assert updated["data"]["goal_metrics"][0]["name"] == "cycle_time"
    assert "goal-updated" in [event["action"] for event in evidence["data"]["events"]]
