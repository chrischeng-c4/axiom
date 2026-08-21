# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_patterns_may_only_match_literals_and_attribute_lookups_1"
# subject = "cpython.test_patma.TestSyntaxErrors.test_patterns_may_only_match_literals_and_attribute_lookups_1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_patterns_may_only_match_literals_and_attribute_lookups_1
"""Auto-ported test: TestSyntaxErrors::test_patterns_may_only_match_literals_and_attribute_lookups_1 (CPython 3.12 oracle)."""


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
assert_syntax_error('\n        match ...:\n            case f"{x}":\n                pass\n        ')
print("TestSyntaxErrors::test_patterns_may_only_match_literals_and_attribute_lookups_1: ok")
