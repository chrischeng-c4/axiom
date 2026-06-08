# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "rlcompleter"
# dimension = "behavior"
# case = "test_rlcompleter__test_namespace"
# subject = "cpython.test_rlcompleter.TestRlcompleter.test_namespace"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_rlcompleter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_rlcompleter.py::TestRlcompleter::test_namespace
"""Auto-ported test: TestRlcompleter::test_namespace (CPython 3.12 oracle)."""


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

class A(dict):
    pass

class B(list):
    pass

assert self_stdcompleter.use_main_ns

assert not self_completer.use_main_ns

assert not rlcompleter.Completer(A()).use_main_ns

try:
    rlcompleter.Completer(B((1,)))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestRlcompleter::test_namespace: ok")
