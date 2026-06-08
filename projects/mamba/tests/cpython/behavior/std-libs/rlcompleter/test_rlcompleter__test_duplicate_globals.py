# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "rlcompleter"
# dimension = "behavior"
# case = "test_rlcompleter__test_duplicate_globals"
# subject = "cpython.test_rlcompleter.TestRlcompleter.test_duplicate_globals"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_rlcompleter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_rlcompleter.py::TestRlcompleter::test_duplicate_globals
"""Auto-ported test: TestRlcompleter::test_duplicate_globals (CPython 3.12 oracle)."""


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
namespace = {'False': None, 'assert': None, 'try': lambda: None, 'memoryview': None, 'Ellipsis': lambda: None}
completer = rlcompleter.Completer(namespace)

assert completer.complete('False', 0) == 'False'

assert completer.complete('False', 1) is None

assert completer.complete('assert', 0) == 'assert '

assert completer.complete('assert', 1) is None

assert completer.complete('try', 0) == 'try:'

assert completer.complete('try', 1) is None

assert completer.complete('memoryview', 0) == 'memoryview'

assert completer.complete('memoryview', 1) is None

assert completer.complete('Ellipsis', 0) == 'Ellipsis()'

assert completer.complete('Ellipsis', 1) is None
print("TestRlcompleter::test_duplicate_globals: ok")
