# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_real_number_required_in_complex_literal_3"
# subject = "cpython.test_patma.TestSyntaxErrors.test_real_number_required_in_complex_literal_3"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_real_number_required_in_complex_literal_3
"""Auto-ported test: TestSyntaxErrors::test_real_number_required_in_complex_literal_3 (CPython 3.12 oracle)."""


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
assert_syntax_error('\n        match ...:\n            case {0j+0j: _}:\n                pass\n        ')
print("TestSyntaxErrors::test_real_number_required_in_complex_literal_3: ok")
