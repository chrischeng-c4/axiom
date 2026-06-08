# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_1"
# subject = "cpython.test_patma.TestSyntaxErrors.test_name_capture_makes_remaining_patterns_unreachable_1"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_1
"""Auto-ported test: TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_1 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match 42:\n            case x:\n                pass\n            case y:\n                pass\n        ')
print("TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_1: ok")
