# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "rlcompleter"
# dimension = "behavior"
# case = "test_rlcompleter__test_global_matches"
# subject = "cpython.test_rlcompleter.TestRlcompleter.test_global_matches"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_rlcompleter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_rlcompleter.py::TestRlcompleter::test_global_matches
"""Auto-ported test: TestRlcompleter::test_global_matches (CPython 3.12 oracle)."""


import unittest
from unittest.mock import patch
import builtins
import rlcompleter
from test.support import MISSING_C_DOCSTRINGS


class CompleteMe:
    """ Trivial class used in testing rlcompleter.Completer. """
    spam = 1
    _ham = 2


# --- test body ---
self_stdcompleter = rlcompleter.Completer()
self_completer = rlcompleter.Completer(dict(spam=int, egg=str, CompleteMe=CompleteMe))
self_stdcompleter.complete('', 0)

assert sorted(self_stdcompleter.global_matches('di')) == [x + '(' for x in dir(builtins) if x.startswith('di')]

assert sorted(self_stdcompleter.global_matches('st')) == [x + '(' for x in dir(builtins) if x.startswith('st')]

assert self_stdcompleter.global_matches('akaksajadhak') == []

assert self_completer.global_matches('CompleteM') == ['CompleteMe(' if MISSING_C_DOCSTRINGS else 'CompleteMe()']

assert self_completer.global_matches('eg') == ['egg(']

assert self_completer.global_matches('CompleteM') == ['CompleteMe(' if MISSING_C_DOCSTRINGS else 'CompleteMe()']
print("TestRlcompleter::test_global_matches: ok")
