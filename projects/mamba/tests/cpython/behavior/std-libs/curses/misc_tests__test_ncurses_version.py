# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses"
# dimension = "behavior"
# case = "misc_tests__test_ncurses_version"
# subject = "cpython.test_curses.MiscTests.test_ncurses_version"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_curses.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_curses.py::MiscTests::test_ncurses_version
"""Auto-ported test: MiscTests::test_ncurses_version (CPython 3.12 oracle)."""


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
v = curses.ncurses_version
if verbose:
    print(f'ncurses_version = {curses.ncurses_version}', flush=True)

assert isinstance(v[:], tuple)

assert len(v) == 3

assert isinstance(v[0], int)

assert isinstance(v[1], int)

assert isinstance(v[2], int)

assert isinstance(v.major, int)

assert isinstance(v.minor, int)

assert isinstance(v.patch, int)

assert v[0] == v.major

assert v[1] == v.minor

assert v[2] == v.patch

assert v.major >= 0

assert v.minor >= 0

assert v.patch >= 0
print("MiscTests::test_ncurses_version: ok")
