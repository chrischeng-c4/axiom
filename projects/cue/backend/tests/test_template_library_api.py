"""Contract tests for Cue MVP template library and pilot dashboard."""

from __future__ import annotations

import asyncio

from test_workstream_api import _load_main


def test_template_library_exposes_seedable_app_spec_examples() -> None:
    main = _load_main()
    handler = main.app._handlers[("GET", "/api/admin/template-library")]

    payload = asyncio.run(handler())

    assert payload["ok"] is True
    library = payload["data"]
    assert library["schema_version"] == "cue.template-library.v0"
    assert library["template_contract"]["seed_targets"] == ["prompt_builder", "app_studio"]
    assert [template["name"] for template in library["templates"]] == [
        "Vendor Onboarding Tracker",
        "Marketing Campaign Approval",
        "Customer Escalation Tracker",
        "Warehouse Incident Tracker",
        "Data Request Intake",
        "Release Readiness Tracker",
        "Weekly Status Collection",
    ]

    required = set(library["template_contract"]["required_fields"])
    for template in library["templates"]:
        assert required <= set(template)
        assert template["seed_targets"] == ["prompt_builder", "app_studio"]
        assert template["app_spec"]["schema_version"] == "cue.app-spec.v0"
        assert template["app_spec"]["app_id"] == template["id"]
        assert template["owner_role"]
        assert template["data_needs"]
        assert template["risk_assumptions"]
        assert template["permissions"]["roles"]
        assert template["tests"]
        assert template["approval_path"]


def test_pilot_acceptance_dashboard_rolls_up_required_metrics() -> None:
    main = _load_main()
    handler = main.app._handlers[("GET", "/api/admin/pilot-acceptance-dashboard")]

    payload = asyncio.run(handler())

    assert payload["ok"] is True
    dashboard = payload["data"]
    assert dashboard["schema_version"] == "cue.pilot-acceptance-dashboard.v0"
    assert dashboard["metrics"]["active_apps"] == 7
    assert dashboard["metrics"]["active_users"] == 184
    assert dashboard["metrics"]["policy_pass_rate"] == 1.0
    assert dashboard["metrics"]["test_pass_rate"] == 1.0
    assert dashboard["metrics"]["deployment_status"]["sandbox"] == 5
    assert dashboard["metrics"]["deployment_status"]["pilot"] == 1
    assert dashboard["metrics"]["deployment_status"]["production_candidate"] == 1
    assert dashboard["metrics"]["incidents"] == 1
    assert dashboard["metrics"]["app_owner_satisfaction"] == 4.4
