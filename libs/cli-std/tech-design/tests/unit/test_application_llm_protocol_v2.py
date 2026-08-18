from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.application.errors import ProtocolInvalid, StepInvalid
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
    render_document,
    step_json,
    task_json,
    template_placeholders,
    validate_document,
    validate_step,
)
from cli_std.domain.errors import UnknownTopic


class TestApplicationLlmProtocolV2(unittest.TestCase):
    def setUp(self) -> None:
        self.sample_task = Task(
            id="t-id",
            use_when="doing x",
            requires=(),
            reads=(),
            produces=(),
            risk=Risk.LOCAL_WRITE,
            topic="t-topic",
            contract_refs=(),
        )
        self.sample_runbook = Runbook(
            purpose="purpose",
            preconditions=(),
            inputs=(),
            constraints=(),
            steps=(),
            verification=(),
            references=(),
        )

    def test_validate_document_empty_project(self) -> None:
        res = validate_document(
            "  ", [ProtocolTopic(self.sample_task, self.sample_runbook)]
        )
        self.assertIsInstance(res, ProtocolInvalid)

    def test_validate_step_both_command_and_template(self) -> None:
        step = Step(
            id="s1",
            instruction="do x",
            command="echo hi",
            command_template="echo {msg}",
            inputs=(Input("msg", "string", "d", True),),
        )
        res = validate_step(step)
        self.assertIsInstance(res, StepInvalid)
        if isinstance(res, StepInvalid):
            self.assertEqual(res.step_id, "s1")

    def test_validate_step_command_unbound_placeholders(self) -> None:
        s_url = Step(
            id="s_url",
            instruction="run",
            command="run <url>",
            command_template=None,
            inputs=(),
        )
        self.assertIsInstance(validate_step(s_url), StepInvalid)

        s_brace = Step(
            id="s_brace",
            instruction="run",
            command="run {x}",
            command_template=None,
            inputs=(),
        )
        self.assertIsInstance(validate_step(s_brace), StepInvalid)

    def test_validate_step_command_template_inputs_matching(self) -> None:
        s_no_inputs = Step(
            id="s_no_inp",
            instruction="run",
            command=None,
            command_template="run {x}",
            inputs=(),
        )
        self.assertIsInstance(validate_step(s_no_inputs), StepInvalid)

        s_valid = Step(
            id="s_ok",
            instruction="run",
            command=None,
            command_template="run {x} {x}",
            inputs=(Input("x", "string", "desc", True),),
        )
        self.assertIsNone(validate_step(s_valid))

    def test_template_placeholders_repeated_name(self) -> None:
        res = template_placeholders("a {x} b {x}")
        self.assertEqual(res, frozenset({"x"}))

    def test_template_placeholders_unclosed_and_unmatched_braces(self) -> None:
        r1 = template_placeholders("a {x")
        self.assertIsInstance(r1, StepInvalid)
        if isinstance(r1, StepInvalid):
            self.assertIn("unclosed placeholder", r1.reason)

        r2 = template_placeholders("a x}")
        self.assertIsInstance(r2, StepInvalid)
        if isinstance(r2, StepInvalid):
            self.assertIn("closing brace", r2.reason)

    def test_template_placeholders_invalid_and_valid_character_names(self) -> None:
        r_space = template_placeholders("a {bad name}")
        self.assertIsInstance(r_space, StepInvalid)

        r_empty = template_placeholders("a {}")
        self.assertIsInstance(r_empty, StepInvalid)

        r_valid = template_placeholders("{a-b_9}")
        self.assertEqual(r_valid, frozenset({"a-b_9"}))

    def test_validate_document_separate_id_topic_namespaces(self) -> None:
        t1 = Task("x", "use", (), (), (), Risk.INSPECT, "p", ())
        t2 = Task("q", "use", (), (), (), Risk.INSPECT, "x", ())
        topic1 = ProtocolTopic(t1, self.sample_runbook)
        topic2 = ProtocolTopic(t2, self.sample_runbook)

        doc = validate_document("proj", [topic1, topic2])
        self.assertIsInstance(doc, ProtocolDocument)

        dup_id = Task("x", "use", (), (), (), Risk.INSPECT, "other", ())
        res_dup_id = validate_document(
            "proj", [topic1, ProtocolTopic(dup_id, self.sample_runbook)]
        )
        self.assertIsInstance(res_dup_id, ProtocolInvalid)

        dup_topic = Task("other", "use", (), (), (), Risk.INSPECT, "p", ())
        res_dup_topic = validate_document(
            "proj", [topic1, ProtocolTopic(dup_topic, self.sample_runbook)]
        )
        self.assertIsInstance(res_dup_topic, ProtocolInvalid)

    def test_step_json_omitted_null_keys(self) -> None:
        step = Step(
            id="s1",
            instruction="instr",
            command=None,
            command_template=None,
            inputs=(),
        )
        sj = step_json(step)
        self.assertEqual(set(sj.keys()), {"id", "instruction"})

    def test_task_json_risk_value_and_input_json_type_key(self) -> None:
        tj = task_json(self.sample_task)
        self.assertEqual(tj["risk"], "local_write")
        self.assertNotEqual(tj["risk"], "LOCAL_WRITE")

        inp = Input("name", "val_type", "desc", True)
        ij = input_json(inp)
        self.assertIn("type", ij)
        self.assertEqual(ij["type"], "val_type")
        self.assertNotIn("value_type", ij)

    def test_render_document_looks_up_topic_not_id(self) -> None:
        task = Task("local-search", "u", (), (), (), Risk.INSPECT, "search", ())
        pt = ProtocolTopic(task, self.sample_runbook)
        doc = ProtocolDocument("lumen", (pt,))

        res_topic = render_document(doc, "search", Format.MD)
        self.assertIsInstance(res_topic, str)

        res_id = render_document(doc, "local-search", Format.MD)
        self.assertIsInstance(res_id, UnknownTopic)

    def test_render_document_json_envelopes(self) -> None:
        pt = ProtocolTopic(self.sample_task, self.sample_runbook)
        doc = ProtocolDocument("lumen", (pt,))

        res_outline = render_document(doc, "outline", Format.JSON)
        self.assertIsInstance(res_outline, dict)
        if isinstance(res_outline, dict):
            self.assertEqual(res_outline["topic"], "outline")
            self.assertIn("markdown", res_outline)
            self.assertEqual(res_outline["protocol"], PROTOCOL)
            self.assertIn("tasks", res_outline)
            self.assertNotIn("runbook", res_outline)

        res_detail = render_document(doc, "t-topic", Format.JSON)
        self.assertIsInstance(res_detail, dict)
        if isinstance(res_detail, dict):
            self.assertEqual(res_detail["topic"], "t-topic")
            self.assertIn("markdown", res_detail)
            self.assertEqual(res_detail["protocol"], PROTOCOL)
            self.assertIn("task", res_detail)
            self.assertIn("runbook", res_detail)
            self.assertNotIn("tasks", res_detail)

    def test_topic_markdown_v2_empty_preconditions_heading_omission(self) -> None:
        pt = ProtocolTopic(self.sample_task, self.sample_runbook)
        doc = ProtocolDocument("lumen", (pt,))
        md = render_document(doc, "t-topic", Format.MD)
        self.assertIsInstance(md, str)
        if isinstance(md, str):
            self.assertNotIn("## Preconditions", md)

        rb_pre = Runbook("purpose", ("pre-cond",), (), (), (), (), ())
        pt_pre = ProtocolTopic(self.sample_task, rb_pre)
        doc_pre = ProtocolDocument("lumen", (pt_pre,))
        md_pre = render_document(doc_pre, "t-topic", Format.MD)
        self.assertIsInstance(md_pre, str)
        if isinstance(md_pre, str):
            self.assertIn("## Preconditions", md_pre)

    def test_runbook_json_structure(self) -> None:
        from cli_std.application.llm_protocol_v2 import runbook_json
        rj = runbook_json(self.sample_runbook)
        self.assertEqual(rj["purpose"], "purpose")
        self.assertIn("steps", rj)


if __name__ == "__main__":
    unittest.main()
