# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "behavior"
# case = "stats_test_case__test_get_stats_profile"
# subject = "cpython.test_pstats.StatsTestCase.test_get_stats_profile"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pstats.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pstats.py::StatsTestCase::test_get_stats_profile
"""Auto-ported test: StatsTestCase::test_get_stats_profile (CPython 3.12 oracle)."""


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

def pass1():
    pass

def pass2():
    pass

def pass3():
    pass
pr = cProfile.Profile()
pr.enable()
pass1()
pass2()
pass3()
pr.create_stats()
ps = pstats.Stats(pr)
stats_profile = ps.get_stats_profile()
funcs_called = set(stats_profile.func_profiles.keys())

assert 'pass1' in funcs_called

assert 'pass2' in funcs_called

assert 'pass3' in funcs_called
print("StatsTestCase::test_get_stats_profile: ok")
