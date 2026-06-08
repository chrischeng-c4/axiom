# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "behavior"
# case = "add_callers_test_case__test_combine_results"
# subject = "cpython.test_pstats.AddCallersTestCase.test_combine_results"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pstats.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pstats.py::AddCallersTestCase::test_combine_results
"""Auto-ported test: AddCallersTestCase::test_combine_results (CPython 3.12 oracle)."""


import unittest
from test import support
from io import StringIO
from pstats import SortKey
from enum import StrEnum, _test_simple_enum
import pstats
import cProfile


# --- test body ---
target = {'a': (1, 2, 3, 4)}
source = {'a': (1, 2, 3, 4), 'b': (5, 6, 7, 8)}
new_callers = pstats.add_callers(target, source)

assert new_callers == {'a': (2, 4, 6, 8), 'b': (5, 6, 7, 8)}
target = {'a': 1}
source = {'a': 1, 'b': 5}
new_callers = pstats.add_callers(target, source)

assert new_callers == {'a': 2, 'b': 5}
print("AddCallersTestCase::test_combine_results: ok")
