# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "except_star"
# dimension = "behavior"
# case = "test_break_continue_return_in_except_star_block__test_return_in_except_star_block_invalid"
# subject = "cpython.test_except_star.TestBreakContinueReturnInExceptStarBlock.test_return_in_except_star_block_invalid"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_except_star.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_except_star.py::TestBreakContinueReturnInExceptStarBlock::test_return_in_except_star_block_invalid
"""Auto-ported test: TestBreakContinueReturnInExceptStarBlock::test_return_in_except_star_block_invalid (CPython 3.12 oracle)."""


import sys
import unittest
import textwrap
from test.support.testcase import ExceptionIsLikeMixin


# --- test body ---
MSG = "'break', 'continue' and 'return' cannot appear in an except\\* block"

def check_invalid(src):
    try:
        compile(textwrap.dedent(src), '<string>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError as _aR_e:
        import re as _re_aR
        assert _re_aR.search(MSG, str(_aR_e))
check_invalid('\n            def f():\n                try:\n                    raise ValueError\n                except* Exception as e:\n                    return 42\n            ')
check_invalid('\n            def f():\n                try:\n                    pass\n                except* Exception as e:\n                    return 42\n                finally:\n                    finished = True\n            ')
print("TestBreakContinueReturnInExceptStarBlock::test_return_in_except_star_block_invalid: ok")
