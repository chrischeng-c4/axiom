# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "behavior"
# case = "stats_test_case__test_add"
# subject = "cpython.test_pstats.StatsTestCase.test_add"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pstats.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pstats.py::StatsTestCase::test_add
"""Auto-ported test: StatsTestCase::test_add (CPython 3.12 oracle)."""


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
stream = StringIO()
stats = pstats.Stats(stream=stream)
stats.add(self_stats, self_stats)
print("StatsTestCase::test_add: ok")
