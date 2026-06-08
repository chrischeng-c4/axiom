# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_multiple_assignments_to_name_in_pattern_0"
# subject = "cpython.test_patma.TestSyntaxErrors.test_multiple_assignments_to_name_in_pattern_0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_0
"""Auto-ported test: TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_0 (CPython 3.12 oracle)."""


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
assert_syntax_error('\n        match ...:\n            case a, a:\n                pass\n        ')
print("TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_0: ok")
