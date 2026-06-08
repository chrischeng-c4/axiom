# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "behavior"
# case = "stats_test_case__test_sort_stats_int"
# subject = "cpython.test_pstats.StatsTestCase.test_sort_stats_int"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pstats.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pstats.py::StatsTestCase::test_sort_stats_int
"""Auto-ported test: StatsTestCase::test_sort_stats_int (CPython 3.12 oracle)."""


import unittest
from test import support
from io import StringIO
from pstats import SortKey
from enum import StrEnum, _test_simple_enum
import pstats
import cProfile


# --- test body ---
stats_file = support.findfile('pstats.pck')
self_stats = pstats.Stats(stats_file)
valid_args = {-1: 'stdname', 0: 'calls', 1: 'time', 2: 'cumulative'}
for arg_int, arg_str in valid_args.items():
    self_stats.sort_stats(arg_int)

    assert self_stats.sort_type == self_stats.sort_arg_dict_default[arg_str][-1]
print("StatsTestCase::test_sort_stats_int: ok")
