# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses"
# dimension = "behavior"
# case = "test_ascii__test_unctrl"
# subject = "cpython.test_curses.TestAscii.test_unctrl"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_curses.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_curses.py::TestAscii::test_unctrl
"""Auto-ported test: TestAscii::test_unctrl (CPython 3.12 oracle)."""


import functools
import inspect
import os
import string
import sys
import tempfile
import unittest
from unittest.mock import MagicMock
from test.support import requires, verbose, SaveSignals, cpython_only, check_disallow_instantiation, MISSING_C_DOCSTRINGS
from test.support.import_helper import import_module


requires('curses')

curses = import_module('curses')

import_module('curses.ascii')

import_module('curses.textpad')

try:
    import curses.panel
except ImportError:
    pass

def requires_curses_func(name):
    return unittest.skipUnless(hasattr(curses, name), 'requires curses.%s' % name)

def requires_curses_window_meth(name):

    def deco(test):

        @functools.wraps(test)
        def wrapped(self, *args, **kwargs):
            if not hasattr(self.stdscr, name):
                raise unittest.SkipTest('requires curses.window.%s' % name)
            test(self, *args, **kwargs)
        return wrapped
    return deco

def requires_colors(test):

    @functools.wraps(test)
    def wrapped(self, *args, **kwargs):
        if not curses.has_colors():
            self.skipTest('requires colors support')
        curses.start_color()
        test(self, *args, **kwargs)
    return wrapped

term = os.environ.get('TERM')

SHORT_MAX = 32767

def lorem_ipsum(win):
    text = ['Lorem ipsum', 'dolor sit amet,', 'consectetur', 'adipiscing elit,', 'sed do eiusmod', 'tempor incididunt', 'ut labore et', 'dolore magna', 'aliqua.']
    maxy, maxx = win.getmaxyx()
    for y, line in enumerate(text[:maxy]):
        win.addstr(y, 0, line[:maxx - (y == maxy - 1)])


# --- test body ---
unctrl = curses.ascii.unctrl

assert unctrl('a') == 'a'

assert unctrl('A') == 'A'

assert unctrl(';') == ';'

assert unctrl(' ') == ' '

assert unctrl('\x7f') == '^?'

assert unctrl('\n') == '^J'

assert unctrl('\x00') == '^@'

assert unctrl(ord('A')) == 'A'

assert unctrl(ord('\n')) == '^J'

assert unctrl('\x8a') == '!^J'

assert unctrl('Á') == '!A'

assert unctrl(ord('\x8a')) == '!^J'

assert unctrl(ord('Á')) == '!A'
print("TestAscii::test_unctrl: ok")
