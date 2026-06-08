# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "behavior"
# case = "stats_test_case__test_sort_starts_mix"
# subject = "cpython.test_pstats.StatsTestCase.test_sort_starts_mix"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pstats.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pstats.py::StatsTestCase::test_sort_starts_mix
"""Auto-ported test: StatsTestCase::test_sort_starts_mix (CPython 3.12 oracle)."""


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

try:
    self_stats.sort_stats('calls', SortKey.TIME)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    self_stats.sort_stats(SortKey.TIME, 'calls')
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("StatsTestCase::test_sort_starts_mix: ok")
