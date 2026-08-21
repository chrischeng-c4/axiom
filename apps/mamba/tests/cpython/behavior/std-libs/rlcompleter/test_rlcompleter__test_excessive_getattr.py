# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "rlcompleter"
# dimension = "behavior"
# case = "test_rlcompleter__test_excessive_getattr"
# subject = "cpython.test_rlcompleter.TestRlcompleter.test_excessive_getattr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_rlcompleter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_rlcompleter.py::TestRlcompleter::test_excessive_getattr
"""Auto-ported test: TestRlcompleter::test_excessive_getattr (CPython 3.12 oracle)."""


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
'Ensure getattr() is invoked no more than once per attribute'

class Foo:
    calls = 0
    bar = ''

    def __getattribute__(self, name):
        if name == 'bar':
            self.calls += 1
            return None
        return super().__getattribute__(name)
f = Foo()
completer = rlcompleter.Completer(dict(f=f))

assert completer.complete('f.b', 0) == 'f.bar'

assert f.calls == 1
print("TestRlcompleter::test_excessive_getattr: ok")
