# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compile"
# dimension = "behavior"
# case = "test_stack_size_stability__test_for_break_continue_inside_try_finally_block"
# subject = "cpython.test_compile.TestStackSizeStability.test_for_break_continue_inside_try_finally_block"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_compile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_compile.py::TestStackSizeStability::test_for_break_continue_inside_try_finally_block
"""Auto-ported test: TestStackSizeStability::test_for_break_continue_inside_try_finally_block (CPython 3.12 oracle)."""


import dis
import math
import os
import unittest
import sys
import ast
import _ast
import tempfile
import types
import textwrap
import warnings
from test import support
from test.support import script_helper, requires_debug_ranges, run_code, requires_specialization, C_RECURSION_LIMIT
from test.support.os_helper import FakePath


# --- test body ---
def check_stack_size(snippet, async_=False):

    def compile_snippet(i):
        ns = {}
        script = 'def func():\n' + i * snippet
        if async_:
            script = 'async ' + script
        code = compile(script, '<script>', 'exec')
        exec(code, ns, ns)
        return ns['func'].__code__
    sizes = [compile_snippet(i).co_stacksize for i in range(2, 5)]
    if len(set(sizes)) != 1:
        import dis, io
        out = io.StringIO()
        dis.dis(compile_snippet(1), file=out)

        raise AssertionError('stack sizes diverge with # of consecutive snippets: %s\n%s\n%s' % (sizes, snippet, out.getvalue()))
snippet = '\n            for x in y:\n                try:\n                    if z:\n                        break\n                    elif u:\n                        continue\n                    else:\n                        a\n                finally:\n                    f\n            else:\n                b\n            '
check_stack_size(snippet)
print("TestStackSizeStability::test_for_break_continue_inside_try_finally_block: ok")
