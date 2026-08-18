from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.application.errors import ProtocolInvalid
from cli_std.application.llm import (
    Format,
    GeneratedSection,
    ProseSection,
    SectionedTopic,
    Topic,
    assert_topics_render,
    join_sections,
    outline_markdown,
    render,
    render_sectioned,
    render_sections,
)
from cli_std.domain.errors import UnknownTopic


class CountingRenderer:
    def __init__(self) -> None:
        self.counter = 0

    def __call__(self) -> str:
        self.counter += 1
        return f"content-{self.counter}"


class TestApplicationLlm(unittest.TestCase):
    def test_generated_section_render_laziness(self) -> None:
        renderer = CountingRenderer()
        gen_sec = GeneratedSection("g1", renderer)
        topic = SectionedTopic("t1", "summary", (gen_sec,))
        self.assertEqual(renderer.counter, 0)

        secs1 = render_sections(topic)
        self.assertEqual(renderer.counter, 1)
        self.assertEqual(secs1[0].content, "content-1")

        secs2 = render_sections(topic)
        self.assertEqual(renderer.counter, 2)
        self.assertEqual(secs2[0].content, "content-2")

    def test_render_sections_prose_index_derived_from_whole_tuple(self) -> None:
        renderer = CountingRenderer()
        gen_sec = GeneratedSection("g", renderer)
        prose_sec = ProseSection("p")
        topic = SectionedTopic("t", "summary", (gen_sec, prose_sec))

        rendered = render_sections(topic)
        self.assertEqual(len(rendered), 2)
        self.assertEqual(rendered[0].id, "g")
        self.assertEqual(rendered[1].id, "prose-1")

    def test_format_parse_case_insensitivity(self) -> None:
        self.assertEqual(Format.parse("JSON"), Format.JSON)
        self.assertEqual(Format.parse("markdown"), Format.MD)
        self.assertEqual(Format.parse(""), Format.MD)
        self.assertEqual(Format.parse("jsonl"), Format.MD)

    def test_render_unknown_topic_known_list(self) -> None:
        topics = [Topic("t1", "sum1", "body1"), Topic("t2", "sum2", "body2")]
        res = render("lumen", "1.0.0", topics, "nope", Format.MD)
        self.assertIsInstance(res, UnknownTopic)
        if isinstance(res, UnknownTopic):
            self.assertEqual(res.known, ("t1", "t2"))
            self.assertNotIn("outline", res.known)

    def test_render_json_outline_dict_shape(self) -> None:
        topics = [Topic("t1", "sum1", "body1")]
        res = render("lumen", "1.0.0", topics, "outline", Format.JSON)
        self.assertIsInstance(res, dict)
        if isinstance(res, dict):
            self.assertEqual(res["project"], "lumen")
            self.assertEqual(res["version"], "1.0.0")
            self.assertIn("topics", res)
            topics_list = res["topics"]
            self.assertIsInstance(topics_list, list)
            if isinstance(topics_list, list) and topics_list:
                t0 = topics_list[0]
                self.assertIsInstance(t0, dict)
                if isinstance(t0, dict):
                    self.assertEqual(set(t0.keys()), {"id", "summary"})
                    self.assertNotIn("body", t0)

    def test_render_sectioned_json_detail_sections_key(self) -> None:
        sec_topic = SectionedTopic("st1", "sum", (ProseSection("p1"),))
        res_sec = render_sectioned(
            "lumen", "1.0.0", [sec_topic], "st1", Format.JSON
        )
        self.assertIsInstance(res_sec, dict)
        if isinstance(res_sec, dict):
            self.assertIn("sections", res_sec)

        plain_topic = Topic("pt1", "sum", "body")
        res_plain = render("lumen", "1.0.0", [plain_topic], "pt1", Format.JSON)
        self.assertIsInstance(res_plain, dict)
        if isinstance(res_plain, dict):
            self.assertNotIn("sections", res_plain)

    def test_assert_topics_render_duplicate_generated_id(self) -> None:
        renderer = CountingRenderer()
        g1 = GeneratedSection("shared-id", renderer)
        g2 = GeneratedSection("shared-id", renderer)

        t1 = SectionedTopic("t1", "s1", (g1,))
        t2 = SectionedTopic("t2", "s2", (g2,))
        res_dup = assert_topics_render([t1, t2])
        self.assertIsInstance(res_dup, ProtocolInvalid)

        t_prose1 = SectionedTopic("tp1", "s1", (ProseSection("a"),))
        t_prose2 = SectionedTopic("tp2", "s2", (ProseSection("b"),))
        self.assertIsNone(assert_topics_render([t_prose1, t_prose2]))

    def test_assert_topics_render_whitespace_output(self) -> None:
        empty_topic = Topic("t_empty", "s", "   ")
        res = assert_topics_render([empty_topic])
        self.assertIsInstance(res, ProtocolInvalid)
        if isinstance(res, ProtocolInvalid):
            self.assertIn("t_empty", res.reason)

    def test_outline_markdown_formatting(self) -> None:
        md = outline_markdown("lumen", [Topic("t1", "s1", "b1")])
        self.assertIn("# lumen — agent topic outline", md)
        self.assertIn("## Standard agent commands", md)

    def test_render_plain_topic_markdown_and_json(self) -> None:
        t = Topic("t1", "s1", "b1")
        res_md = render("lumen", "1.0.0", [t], "t1", Format.MD)
        self.assertEqual(res_md, "b1")

        res_json = render("lumen", "1.0.0", [t], "t1", Format.JSON)
        self.assertIsInstance(res_json, dict)

    def test_assert_topics_render_empty_generated_section(self) -> None:
        bad_gen = GeneratedSection("g_empty", lambda: "   ")
        t = SectionedTopic("t_gen", "s", (bad_gen,))
        res = assert_topics_render([t])
        self.assertIsInstance(res, ProtocolInvalid)

    def test_join_sections_utility(self) -> None:
        rendered = render_sections(Topic("t", "s", "hello"))
        self.assertEqual(join_sections(rendered), "hello")


if __name__ == "__main__":
    unittest.main()
