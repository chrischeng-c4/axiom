from __future__ import annotations

from cli_std.application.errors import ProtocolInvalid, StepInvalid
from cli_std.application.llm import Format
from cli_std.application.llm_protocol_v2 import (
    Input,
    ProtocolDocument,
    ProtocolTopic,
    Risk,
    Runbook,
    Step,
    Task,
    render_document,
    template_placeholders,
    validate_document,
    validate_step,
)
from cli_std.domain.errors import UnknownTopic

MINIMUM_CHECKS = 14

TYPED_AGENT_PROTOCOL_V2_SECURITY_MATRIX = [
    ("validate_step_both_command_and_template_refused", ("s1", "cannot contain both command and command_template")),
    ("validate_step_bound_command_with_placeholder_refused", ("s1", "command must be fully bound; use command_template with typed inputs")),
    ("validate_step_bound_command_with_inputs_refused", ("s1", "command must be fully bound; use command_template with typed inputs")),
    ("validate_step_template_without_inputs_refused", ("s1", "command_template requires typed inputs")),
    ("validate_step_template_declared_exceeds_referenced_refused", ("s1", "command_template placeholders must exactly match its typed inputs")),
    ("validate_step_template_referenced_exceeds_declared_refused", ("s1", "command_template placeholders must exactly match its typed inputs")),
    ("template_placeholders_unclosed_brace", ("s-malformed", "unclosed placeholder")),
    ("template_placeholders_empty_placeholder", ("s-empty", "invalid placeholder")),
    ("template_placeholders_invalid_character", ("s-invalid-char", "invalid placeholder")),
    ("template_placeholders_stray_closing_brace", ("s-stray", "closing brace")),
    ("validate_document_validations_and_refusals", ("project cannot be empty", "task id and topic are both required", "task id and topic are both required", ("s1", "cannot contain both command and command_template"))),
    ("validate_document_duplicate_task_id_refused", "duplicate task id 't1'"),
    ("validate_document_duplicate_topic_refused", "duplicate topic 'top1'"),
    ("render_document_unknown_topic_lists_all_known_topics", ("unknown", ("top1",))),
]


def verify_typed_agent_protocol_v2_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    res0 = validate_step(Step("s1", "inst", "cmd", "cmd {x}", (Input("x", "string", "desc", True),)))
    c0 = (res0.step_id, res0.reason) if isinstance(res0, StepInvalid) else None
    checks.append({"name": "validate_step_both_command_and_template_refused", "passed": c0 == ("s1", "cannot contain both command and command_template")})

    res1 = validate_step(Step("s1", "inst", "cmd {x}", None, ()))
    c1 = (res1.step_id, res1.reason) if isinstance(res1, StepInvalid) else None
    checks.append({"name": "validate_step_bound_command_with_placeholder_refused", "passed": c1 == ("s1", "command must be fully bound; use command_template with typed inputs")})

    res2 = validate_step(Step("s1", "inst", "cmd", None, (Input("x", "string", "desc", True),)))
    c2 = (res2.step_id, res2.reason) if isinstance(res2, StepInvalid) else None
    checks.append({"name": "validate_step_bound_command_with_inputs_refused", "passed": c2 == ("s1", "command must be fully bound; use command_template with typed inputs")})

    res3 = validate_step(Step("s1", "inst", None, "cmd {x}", ()))
    c3 = (res3.step_id, res3.reason) if isinstance(res3, StepInvalid) else None
    checks.append({"name": "validate_step_template_without_inputs_refused", "passed": c3 == ("s1", "command_template requires typed inputs")})

    res4 = validate_step(Step("s1", "inst", None, "cmd {x}", (Input("x", "string", "d", True), Input("y", "string", "d", True))))
    c4 = (res4.step_id, res4.reason) if isinstance(res4, StepInvalid) else None
    checks.append({"name": "validate_step_template_declared_exceeds_referenced_refused", "passed": c4 == ("s1", "command_template placeholders must exactly match its typed inputs")})

    res5 = validate_step(Step("s1", "inst", None, "cmd {x} {y}", (Input("x", "string", "d", True),)))
    c5 = (res5.step_id, res5.reason) if isinstance(res5, StepInvalid) else None
    checks.append({"name": "validate_step_template_referenced_exceeds_declared_refused", "passed": c5 == ("s1", "command_template placeholders must exactly match its typed inputs")})

    res6 = validate_step(Step("s-malformed", "inst", None, "cmd {x", (Input("x", "string", "d", True),)))
    c6 = (res6.step_id, res6.reason) if isinstance(res6, StepInvalid) else None
    checks.append({"name": "template_placeholders_unclosed_brace", "passed": c6 == ("s-malformed", "unclosed placeholder")})

    res7 = validate_step(Step("s-empty", "inst", None, "cmd {}", (Input("x", "string", "d", True),)))
    c7 = (res7.step_id, res7.reason) if isinstance(res7, StepInvalid) else None
    checks.append({"name": "template_placeholders_empty_placeholder", "passed": c7 == ("s-empty", "invalid placeholder")})

    res8 = validate_step(Step("s-invalid-char", "inst", None, "cmd {x.y}", (Input("x", "string", "d", True),)))
    c8 = (res8.step_id, res8.reason) if isinstance(res8, StepInvalid) else None
    checks.append({"name": "template_placeholders_invalid_character", "passed": c8 == ("s-invalid-char", "invalid placeholder")})

    res9 = validate_step(Step("s-stray", "inst", None, "cmd x}", (Input("x", "string", "d", True),)))
    c9 = (res9.step_id, res9.reason) if isinstance(res9, StepInvalid) else None
    checks.append({"name": "template_placeholders_stray_closing_brace", "passed": c9 == ("s-stray", "closing brace")})

    t1 = Task("t1", "when", (), (), (), Risk.INSPECT, "top1", ())
    rb1 = Runbook("p", (), (), (), (Step("s1", "inst", "cmd", None, ()),), (), ())

    res10_a = validate_document("   ", (ProtocolTopic(t1, rb1),))
    reason10_a = res10_a.reason if isinstance(res10_a, ProtocolInvalid) else None

    t_blank_id = Task("  ", "when", (), (), (), Risk.INSPECT, "top1", ())
    res10_b = validate_document("proj", (ProtocolTopic(t_blank_id, rb1),))
    reason10_b = res10_b.reason if isinstance(res10_b, ProtocolInvalid) else None

    t_blank_topic = Task("t1", "when", (), (), (), Risk.INSPECT, "  ", ())
    res10_c = validate_document("proj", (ProtocolTopic(t_blank_topic, rb1),))
    reason10_c = res10_c.reason if isinstance(res10_c, ProtocolInvalid) else None

    step_inv = Step("s1", "inst", "cmd", "cmd {x}", (Input("x", "string", "d", True),))
    rb_inv = Runbook("p", (), (), (), (step_inv,), (), ())
    t_inv = Task("t_inv", "when", (), (), (), Risk.INSPECT, "top_inv", ())
    res10_d = validate_document("proj", (ProtocolTopic(t_inv, rb_inv),))
    tuple10_d = (res10_d.step_id, res10_d.reason) if isinstance(res10_d, StepInvalid) else None

    expected_doc_validations = (
        "project cannot be empty",
        "task id and topic are both required",
        "task id and topic are both required",
        ("s1", "cannot contain both command and command_template"),
    )
    checks.append({"name": "validate_document_validations_and_refusals", "passed": (reason10_a, reason10_b, reason10_c, tuple10_d) == expected_doc_validations})

    t1_dup = Task("t1", "when2", (), (), (), Risk.INSPECT, "top2", ())
    res11 = validate_document("proj", (ProtocolTopic(t1, rb1), ProtocolTopic(t1_dup, rb1)))
    c11 = res11.reason if isinstance(res11, ProtocolInvalid) else None
    checks.append({"name": "validate_document_duplicate_task_id_refused", "passed": c11 == "duplicate task id 't1'"})

    t2_dup_top = Task("t2", "when2", (), (), (), Risk.INSPECT, "top1", ())
    res12 = validate_document("proj", (ProtocolTopic(t1, rb1), ProtocolTopic(t2_dup_top, rb1)))
    c12 = res12.reason if isinstance(res12, ProtocolInvalid) else None
    checks.append({"name": "validate_document_duplicate_topic_refused", "passed": c12 == "duplicate topic 'top1'"})

    doc_valid = ProtocolDocument("proj", (ProtocolTopic(t1, rb1),))
    res13 = render_document(doc_valid, "unknown", Format.MD)
    c13 = (res13.topic, res13.known) if isinstance(res13, UnknownTopic) else None
    checks.append({"name": "render_document_unknown_topic_lists_all_known_topics", "passed": c13 == ("unknown", ("top1",))})

    return {
        "case_id": "typed-agent-protocol-v2-security",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
