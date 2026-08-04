from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.domain.chainable import (
    JsonValue,
    assert_chainable,
    has_runnable_command,
    has_terminal_marker,
)


class RecordingParser:
    def __init__(self, return_val: JsonValue | None = None) -> None:
        self.calls: list[str] = []
        self.return_val = return_val

    def __call__(self, text: str) -> JsonValue | None:
        self.calls.append(text)
        return self.return_val


class TestDomainChainable(unittest.TestCase):
    def test_assert_chainable_accepted_shapes(self) -> None:
        shapes = [
            {"invoke": {"command": "aw td"}},
            {"next": {"command": "aw wi"}},
            {"next": "done"},
        ]
        for shape in shapes:
            parser = RecordingParser(shape)
            res = assert_chainable('{"some": "json"}', parser)
            self.assertIsNone(res)

        parser_none = RecordingParser(None)
        res_text = assert_chainable("next: aw td", parser_none)
        self.assertIsNone(res_text)

    def test_assert_chainable_terminal_completion(self) -> None:
        shape = {"completion": {"workflow_complete": True}}
        parser = RecordingParser(shape)
        res = assert_chainable('{"completion": ...}', parser)
        self.assertIsNone(res)

    def test_has_terminal_marker_non_bool_int(self) -> None:
        payload = {"completion": {"workflow_complete": 1}}
        self.assertFalse(has_terminal_marker(payload))

    def test_assert_chainable_empty_output(self) -> None:
        recorder = RecordingParser()
        res = assert_chainable("   ", recorder)
        self.assertIsNotNone(res)
        self.assertIn("empty", res.reason)
        self.assertEqual(recorder.calls, [])

    def test_assert_chainable_json_without_command_or_marker(self) -> None:
        parser = RecordingParser({"key": "val"})
        res = assert_chainable('{"key": "val"}', parser)
        self.assertIsNotNone(res)
        self.assertIn("terminal marker", res.reason)

    def test_assert_chainable_next_string_cases(self) -> None:
        parser_terminal = RecordingParser({"next": "done"})
        self.assertIsNone(assert_chainable('{"next": "done"}', parser_terminal))

        parser_empty = RecordingParser({"next": ""})
        v1 = assert_chainable('{"next": ""}', parser_empty)
        self.assertIsNotNone(v1)

        parser_blank = RecordingParser({"next": "   "})
        v2 = assert_chainable('{"next": "   "}', parser_blank)
        self.assertIsNotNone(v2)

    def test_assert_chainable_next_prefix_violations(self) -> None:
        parser = RecordingParser(None)
        res1 = assert_chainable("next:done", parser)
        self.assertIsNotNone(res1)
        self.assertIn("next:", res1.reason)

        res2 = assert_chainable("next: ", parser)
        self.assertIsNotNone(res2)
        self.assertIn("next:", res2.reason)

    def test_assert_chainable_trailing_lines(self) -> None:
        parser = RecordingParser(None)
        res_valid = assert_chainable("something\nnext: aw td\n\n\n", parser)
        self.assertIsNone(res_valid)

        res_invalid = assert_chainable("next: aw td\nother non empty", parser)
        self.assertIsNotNone(res_invalid)

    def test_has_runnable_command_robustness(self) -> None:
        self.assertFalse(has_runnable_command({"invoke": "aw td"}))
        self.assertFalse(has_runnable_command({"next": {"kind": "inspect_parent"}}))


if __name__ == "__main__":
    unittest.main()
