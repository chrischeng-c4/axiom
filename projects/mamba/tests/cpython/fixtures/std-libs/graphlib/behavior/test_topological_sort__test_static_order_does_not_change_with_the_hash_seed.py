# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "graphlib"
# dimension = "behavior"
# case = "test_topological_sort__test_static_order_does_not_change_with_the_hash_seed"
# subject = "cpython.test_graphlib.TestTopologicalSort.test_static_order_does_not_change_with_the_hash_seed"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_graphlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_graphlib.py::TestTopologicalSort::test_static_order_does_not_change_with_the_hash_seed
"""Auto-ported test: TestTopologicalSort::test_static_order_does_not_change_with_the_hash_seed (CPython 3.12 oracle)."""


import graphlib
import os
import unittest
from test.support.script_helper import assert_python_ok


# --- test body ---
def check_order_with_hash_seed(seed):
    code = "if 1:\n                import graphlib\n                ts = graphlib.TopologicalSorter()\n                ts.add('blech', 'bluch', 'hola')\n                ts.add('abcd', 'blech', 'bluch', 'a', 'b')\n                ts.add('a', 'a string', 'something', 'b')\n                ts.add('bluch', 'hola', 'abcde', 'a', 'b')\n                print(list(ts.static_order()))\n                "
    env = os.environ.copy()
    env['__cleanenv'] = True
    env['PYTHONHASHSEED'] = str(seed)
    out = assert_python_ok('-c', code, **env)
    return out
run1 = check_order_with_hash_seed(1234)
run2 = check_order_with_hash_seed(31415)

assert run1 != ''

assert run2 != ''

assert run1 == run2
print("TestTopologicalSort::test_static_order_does_not_change_with_the_hash_seed: ok")
