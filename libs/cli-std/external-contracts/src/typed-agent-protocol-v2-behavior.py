from __future__ import annotations

from cli_std.application.llm import Format
from cli_std.application.llm_protocol_v2 import (
    PROTOCOL,
    Input,
    ProtocolDocument,
    ProtocolTopic,
    Risk,
    Runbook,
    Step,
    Task,
    input_json,
    outline_markdown_v2,
    render_document,
    runbook_json,
    step_json,
    task_json,
    topic_markdown_v2,
)

MINIMUM_CHECKS = 13

TYPED_AGENT_PROTOCOL_V2_BEHAVIOR_MATRIX = [
    ("protocol_constant_value", "cclab.llm.v2"),
    ("risk_enum_wire_values", ("inspect", "local_write")),
    ("task_json_serialization", {"id": "t1", "use_when": "when", "requires": ["req"], "reads": ["rd"], "produces": ["prod"], "risk": "inspect", "topic": "top", "contract_refs": ["ref"]}),
    ("input_json_type_key_and_requiredness", {"name": "i1", "type": "string", "description": "desc", "required": True}),
    ("step_json_omits_absent_command_and_empty_inputs", {"id": "s1", "instruction": "do s1", "command": "cmd1"}),
    ("step_json_with_template_and_inputs", {"id": "s2", "instruction": "do s2", "command_template": "cmd {x}", "inputs": [{"name": "x", "type": "string", "description": "desc", "required": True}]}),
    ("topic_markdown_v2_step_numbering_starts_at_one", "# proj — t1\n\npurpose\n\n## Steps\n\n1. do s1\n   `cmd1`\n2. do s2\n   Template: `cmd {x}`"),
    ("topic_markdown_v2_renders_optional_inputs", "# proj — t1\n\npurpose\n\n## Inputs\n\n- `opt` (string, optional) — desc\n\n## Steps\n\n1. do s1\n   `cmd1`"),
    ("render_document_json_includes_protocol_task_runbook", {"topic": "top", "markdown": "# proj — t1\n\npurpose\n\n## Steps\n\n1. do s1\n   `cmd1`", "protocol": "cclab.llm.v2", "task": {"id": "t1", "use_when": "when", "requires": [], "reads": [], "produces": [], "risk": "inspect", "topic": "top", "contract_refs": []}, "runbook": {"purpose": "purpose", "preconditions": [], "inputs": [], "constraints": [], "steps": [{"id": "s1", "instruction": "do s1", "command": "cmd1"}], "verification": [], "references": []}}),
    ("runbook_json_structure", {"purpose": "p", "preconditions": [], "inputs": [], "constraints": [], "steps": [{"id": "s1", "instruction": "do s1", "command": "cmd1"}], "verification": [], "references": []}),
    ("render_document_outline_json", {"topic": "outline", "markdown": "# proj task navigation\n\nSelect the smallest task, then read its typed runbook:\n\n- `t1` — when (`proj llm --topic top`)\n\nUse `proj llm --topic <topic> --format json` for cclab.llm.v2 data.", "protocol": "cclab.llm.v2", "tasks": [{"id": "t1", "use_when": "when", "requires": [], "reads": [], "produces": [], "risk": "inspect", "topic": "top", "contract_refs": []}]}),
    ("outline_markdown_v2_format", "# proj task navigation\n\nSelect the smallest task, then read its typed runbook:\n\n- `t1` — when (`proj llm --topic top`)\n\nUse `proj llm --topic <topic> --format json` for cclab.llm.v2 data."),
    ("render_document_markdown_detail", "# proj — t1\n\npurpose\n\n## Steps\n\n1. do s1\n   `cmd1`"),
]


def verify_typed_agent_protocol_v2_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    c0 = PROTOCOL
    checks.append({"name": "protocol_constant_value", "passed": c0 == "cclab.llm.v2"})

    c1 = (Risk.INSPECT.value, Risk.LOCAL_WRITE.value)
    checks.append({"name": "risk_enum_wire_values", "passed": c1 == ("inspect", "local_write")})

    t1 = Task("t1", "when", ("req",), ("rd",), ("prod",), Risk.INSPECT, "top", ("ref",))
    c2 = task_json(t1)
    checks.append({"name": "task_json_serialization", "passed": c2 == {"id": "t1", "use_when": "when", "requires": ["req"], "reads": ["rd"], "produces": ["prod"], "risk": "inspect", "topic": "top", "contract_refs": ["ref"]}})

    c3 = input_json(Input("i1", "string", "desc", True))
    checks.append({"name": "input_json_type_key_and_requiredness", "passed": c3 == {"name": "i1", "type": "string", "description": "desc", "required": True}})

    s1 = Step("s1", "do s1", "cmd1", None, ())
    c4 = step_json(s1)
    checks.append({"name": "step_json_omits_absent_command_and_empty_inputs", "passed": c4 == {"id": "s1", "instruction": "do s1", "command": "cmd1"}})

    s2 = Step("s2", "do s2", None, "cmd {x}", (Input("x", "string", "desc", True),))
    c5 = step_json(s2)
    checks.append({"name": "step_json_with_template_and_inputs", "passed": c5 == {"id": "s2", "instruction": "do s2", "command_template": "cmd {x}", "inputs": [{"name": "x", "type": "string", "description": "desc", "required": True}]}})

    rb = Runbook("purpose", (), (), (), (s1, s2), (), ())
    t_simple = Task("t1", "when", (), (), (), Risk.INSPECT, "top", ())
    topic_entry = ProtocolTopic(t_simple, rb)
    doc = ProtocolDocument("proj", (topic_entry,))

    c6 = topic_markdown_v2(doc, topic_entry)
    checks.append({"name": "topic_markdown_v2_step_numbering_starts_at_one", "passed": c6 == "# proj — t1\n\npurpose\n\n## Steps\n\n1. do s1\n   `cmd1`\n2. do s2\n   Template: `cmd {x}`"})

    rb_opt = Runbook("purpose", (), (Input("opt", "string", "desc", False),), (), (s1,), (), ())
    topic_opt_entry = ProtocolTopic(t_simple, rb_opt)
    doc_opt = ProtocolDocument("proj", (topic_opt_entry,))
    c7 = topic_markdown_v2(doc_opt, topic_opt_entry)
    checks.append({"name": "topic_markdown_v2_renders_optional_inputs", "passed": c7 == "# proj — t1\n\npurpose\n\n## Inputs\n\n- `opt` (string, optional) — desc\n\n## Steps\n\n1. do s1\n   `cmd1`"})

    rb_simple = Runbook("purpose", (), (), (), (s1,), (), ())
    topic_simple_entry = ProtocolTopic(t_simple, rb_simple)
    doc_simple = ProtocolDocument("proj", (topic_simple_entry,))
    c8 = render_document(doc_simple, "top", Format.JSON)
    expected_doc_json = {"topic": "top", "markdown": "# proj — t1\n\npurpose\n\n## Steps\n\n1. do s1\n   `cmd1`", "protocol": "cclab.llm.v2", "task": {"id": "t1", "use_when": "when", "requires": [], "reads": [], "produces": [], "risk": "inspect", "topic": "top", "contract_refs": []}, "runbook": {"purpose": "purpose", "preconditions": [], "inputs": [], "constraints": [], "steps": [{"id": "s1", "instruction": "do s1", "command": "cmd1"}], "verification": [], "references": []}}
    checks.append({"name": "render_document_json_includes_protocol_task_runbook", "passed": c8 == expected_doc_json})

    c9 = runbook_json(Runbook("p", (), (), (), (s1,), (), ()))
    checks.append({"name": "runbook_json_structure", "passed": c9 == {"purpose": "p", "preconditions": [], "inputs": [], "constraints": [], "steps": [{"id": "s1", "instruction": "do s1", "command": "cmd1"}], "verification": [], "references": []}})

    c10 = render_document(doc_simple, "outline", Format.JSON)
    expected_outline_json = {"topic": "outline", "markdown": "# proj task navigation\n\nSelect the smallest task, then read its typed runbook:\n\n- `t1` — when (`proj llm --topic top`)\n\nUse `proj llm --topic <topic> --format json` for cclab.llm.v2 data.", "protocol": "cclab.llm.v2", "tasks": [{"id": "t1", "use_when": "when", "requires": [], "reads": [], "produces": [], "risk": "inspect", "topic": "top", "contract_refs": []}]}
    checks.append({"name": "render_document_outline_json", "passed": c10 == expected_outline_json})

    c11 = outline_markdown_v2(doc_simple)
    checks.append({"name": "outline_markdown_v2_format", "passed": c11 == "# proj task navigation\n\nSelect the smallest task, then read its typed runbook:\n\n- `t1` — when (`proj llm --topic top`)\n\nUse `proj llm --topic <topic> --format json` for cclab.llm.v2 data."})

    c12 = render_document(doc_simple, "top", Format.MD)
    expected_detail_md = "# proj — t1\n\npurpose\n\n## Steps\n\n1. do s1\n   `cmd1`"
    checks.append({"name": "render_document_markdown_detail", "passed": c12 == expected_detail_md})

    return {
        "case_id": "typed-agent-protocol-v2-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
