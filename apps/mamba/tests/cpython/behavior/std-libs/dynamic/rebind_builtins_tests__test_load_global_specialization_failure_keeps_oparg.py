# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dynamic"
# dimension = "behavior"
# case = "rebind_builtins_tests__test_load_global_specialization_failure_keeps_oparg"
# subject = "cpython.test_dynamic.RebindBuiltinsTests.test_load_global_specialization_failure_keeps_oparg"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_dynamic.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_dynamic.py::RebindBuiltinsTests::test_load_global_specialization_failure_keeps_oparg
"""Auto-ported test: RebindBuiltinsTests::test_load_global_specialization_failure_keeps_oparg (CPython 3.12 oracle)."""


import builtins
import sys
import unittest
from test.support import swap_item, swap_attr


# --- test body ---
class MyGlobals(dict):

    def __missing__(self, key):
        return int(key.removeprefix('_number_'))
variables = 400
code = 'lambda: ' + '+'.join((f'_number_{i}' for i in range(variables)))
sum_func = eval(code, MyGlobals())
expected = sum(range(variables))
for _ in range(30):

    assert sum_func() == expected
print("RebindBuiltinsTests::test_load_global_specialization_failure_keeps_oparg: ok")
