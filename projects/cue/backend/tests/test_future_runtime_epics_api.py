"""Contract tests for post-Tracker Triage and Approval runtime epics."""

from __future__ import annotations

import asyncio

from test_workstream_api import _load_main


def test_triage_runtime_sandbox_assigns_escalates_and_audits_queue_items() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/admin/triage/runtime-sandbox")]
    evidence_handler = main.app._handlers[("GET", "/api/admin/workitems/{workitem_id}/evidence")]

    payload = asyncio.run(
        handler(
            {
                "items": [
                    {"id": "triage-1", "priority": "critical", "impact": "warehouse_down", "age_hours": 6},
                    {"id": "triage-2", "priority": "normal", "impact": "question", "age_hours": 4},
                ]
            }
        )
    )
    evidence = asyncio.run(evidence_handler("request-tracker-prd"))

    assert payload["data"]["schema_version"] == "cue.triage-runtime-sandbox.v0"
    assert payload["data"]["app_spec_lifecycle"][0] == "app_spec"
    assert payload["data"]["queue_view"][0]["assignee"] == "incident_manager"
    assert payload["data"]["queue_view"][0]["sla_status"] == "breached"
    assert payload["data"]["dashboard"]["sla_breaches"] == 1
    assert {test["id"] for test in payload["data"]["tests"]} >= {
        "assignment-rules",
        "escalation-rules",
        "sla-dashboard",
        "audit-trace",
    }
    assert "triage-assigned" in [event["action"] for event in evidence["data"]["events"]]


def test_approval_runtime_sandbox_routes_decisions_without_bypassing_gates() -> None:
    main = _load_main()
    handler = main.app._handlers[("POST", "/api/admin/approval/runtime-sandbox")]
    evidence_handler = main.app._handlers[("GET", "/api/admin/workitems/{workitem_id}/evidence")]

    payload = asyncio.run(
        handler(
            {
                "role": "approver",
                "request": {"id": "approval-1", "amount": 2200, "risk_tier": "tier_2"},
            }
        )
    )
    evidence = asyncio.run(evidence_handler("request-tracker-prd"))

    node_ids = {node["id"] for node in payload["data"]["selected_nodes"]}
    assert payload["data"]["schema_version"] == "cue.approval-runtime-sandbox.v0"
    assert {"manager", "data_owner"} <= node_ids
    assert payload["data"]["permission_check"]["allowed"] is True
    assert payload["data"]["risk"]["required_approvals"] == ["app_owner"]
    assert {decision["decision"] for decision in payload["data"]["decisions"]} == {"approved"}
    assert {test["id"] for test in payload["data"]["tests"]} >= {
        "conditional-routing",
        "approval-audit",
        "data-owner-security-gates",
    }
    assert "approval-decision-recorded" in [event["action"] for event in evidence["data"]["events"]]
