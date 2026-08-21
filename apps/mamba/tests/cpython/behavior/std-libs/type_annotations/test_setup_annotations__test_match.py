# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "type_annotations"
# dimension = "behavior"
# case = "test_setup_annotations__test_match"
# subject = "cpython.test_type_annotations.TestSetupAnnotations.test_match"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_type_annotations.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_type_annotations.py::TestSetupAnnotations::test_match
"""Auto-ported test: TestSetupAnnotations::test_match (CPython 3.12 oracle)."""


import textwrap
import unittest
from test.support import run_code


# --- test body ---
def check(code: str):
    code = textwrap.dedent(code)
    for scope in ('module', 'class'):
        if scope == 'class':
            code = f"class C:\n{textwrap.indent(code, '    ')}"
        ns = run_code(code)
        if scope == 'class':
            annotations = ns['C'].__annotations__
        else:
            annotations = ns['__annotations__']

        assert annotations == {'x': int}
check('\n            match 0:\n                case 0:\n                    x: int = 1\n        ')
print("TestSetupAnnotations::test_match: ok")
