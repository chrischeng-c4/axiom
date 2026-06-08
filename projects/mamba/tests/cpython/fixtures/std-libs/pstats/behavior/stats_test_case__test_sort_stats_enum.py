# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "behavior"
# case = "stats_test_case__test_sort_stats_enum"
# subject = "cpython.test_pstats.StatsTestCase.test_sort_stats_enum"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pstats.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pstats.py::StatsTestCase::test_sort_stats_enum
"""Auto-ported test: StatsTestCase::test_sort_stats_enum (CPython 3.12 oracle)."""


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
for member in SortKey:
    self_stats.sort_stats(member)

    assert self_stats.sort_type == self_stats.sort_arg_dict_default[member.value][-1]

class CheckedSortKey(StrEnum):
    CALLS = ('calls', 'ncalls')
    CUMULATIVE = ('cumulative', 'cumtime')
    FILENAME = ('filename', 'module')
    LINE = 'line'
    NAME = 'name'
    NFL = 'nfl'
    PCALLS = 'pcalls'
    STDNAME = 'stdname'
    TIME = ('time', 'tottime')

    def __new__(cls, *values):
        value = values[0]
        obj = str.__new__(cls, value)
        obj._value_ = value
        for other_value in values[1:]:
            cls._value2member_map_[other_value] = obj
        obj._all_values = values
        return obj
_test_simple_enum(CheckedSortKey, SortKey)
print("StatsTestCase::test_sort_stats_enum: ok")
