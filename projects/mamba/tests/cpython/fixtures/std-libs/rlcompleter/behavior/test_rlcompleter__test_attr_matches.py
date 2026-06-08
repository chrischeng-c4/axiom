# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "rlcompleter"
# dimension = "behavior"
# case = "test_rlcompleter__test_attr_matches"
# subject = "cpython.test_rlcompleter.TestRlcompleter.test_attr_matches"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_rlcompleter.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_rlcompleter.py::TestRlcompleter::test_attr_matches
"""Auto-ported test: TestRlcompleter::test_attr_matches (CPython 3.12 oracle)."""


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

assert self_stdcompleter.attr_matches('str.s') == ['str.{}('.format(x) for x in dir(str) if x.startswith('s')]

assert self_stdcompleter.attr_matches('tuple.foospamegg') == []
expected = sorted({'None.%s%s' % (x, '(' if x != '__doc__' else '') for x in dir(None)})

assert self_stdcompleter.attr_matches('None.') == expected

assert self_stdcompleter.attr_matches('None._') == expected

assert self_stdcompleter.attr_matches('None.__') == expected

assert self_completer.attr_matches('CompleteMe.sp') == ['CompleteMe.spam']

assert self_completer.attr_matches('Completeme.egg') == []

assert self_completer.attr_matches('CompleteMe.') == ['CompleteMe.mro()', 'CompleteMe.spam']

assert self_completer.attr_matches('CompleteMe._') == ['CompleteMe._ham']
matches = self_completer.attr_matches('CompleteMe.__')
for x in matches:

    assert x.startswith('CompleteMe.__')

assert 'CompleteMe.__name__' in matches

assert 'CompleteMe.__new__(' in matches
with patch.object(CompleteMe, 'me', CompleteMe, create=True):

    assert self_completer.attr_matches('CompleteMe.me.me.sp') == ['CompleteMe.me.me.spam']

    assert self_completer.attr_matches('egg.s') == ['egg.{}('.format(x) for x in dir(str) if x.startswith('s')]
print("TestRlcompleter::test_attr_matches: ok")
