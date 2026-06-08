# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "rlcompleter"
# dimension = "behavior"
# case = "test_rlcompleter__test_property_method_not_called"
# subject = "cpython.test_rlcompleter.TestRlcompleter.test_property_method_not_called"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_rlcompleter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_rlcompleter.py::TestRlcompleter::test_property_method_not_called
"""Auto-ported test: TestRlcompleter::test_property_method_not_called (CPython 3.12 oracle)."""


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

class Foo:
    _bar = 0
    property_called = False

    @property
    def bar(self):
        self.property_called = True
        return self._bar
f = Foo()
completer = rlcompleter.Completer(dict(f=f))

assert completer.complete('f.b', 0) == 'f.bar'

assert not f.property_called
print("TestRlcompleter::test_property_method_not_called: ok")
