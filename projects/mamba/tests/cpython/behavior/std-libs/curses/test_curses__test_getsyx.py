# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "curses"
# dimension = "behavior"
# case = "test_curses__test_getsyx"
# subject = "cpython.test_curses.TestCurses.test_getsyx"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_curses.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_curses.py::TestCurses::test_getsyx
"""Auto-ported test: TestCurses::test_getsyx (CPython 3.12 oracle)."""


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
def bad_colors():
    return (-1, curses.COLORS, -2 ** 31 - 1, 2 ** 31, -2 ** 63 - 1, 2 ** 63, 2 ** 64)

def bad_colors2():
    return (curses.COLORS, 2 ** 31, 2 ** 63, 2 ** 64)

def bad_pairs():
    return (-1, -2 ** 31 - 1, 2 ** 31, -2 ** 63 - 1, 2 ** 63, 2 ** 64)

def get_pair_limit():
    pair_limit = curses.COLOR_PAIRS
    if hasattr(curses, 'ncurses_version'):
        if curses.has_extended_color_support():
            pair_limit += 2 * curses.COLORS + 1
        if not curses.has_extended_color_support() or (6, 1) <= curses.ncurses_version < (6, 2):
            pair_limit = min(pair_limit, SHORT_MAX)
        try:
            curses.init_pair(pair_limit - 1, 0, 0)
        except ValueError:
            pair_limit = curses.COLOR_PAIRS
    return pair_limit
self_isatty = True
self_output = sys.__stdout__
stdout_fd = sys.__stdout__.fileno()
if not sys.__stdout__.isatty():
    dup_fd = os.dup(stdout_fd)
    pass
    pass
    if sys.__stderr__.isatty():
        tmp = sys.__stderr__
        self_output = sys.__stderr__
    else:
        try:
            tmp = open('/dev/tty', 'wb', buffering=0)
        except OSError:
            tmp = tempfile.TemporaryFile(mode='wb', buffering=0)
            self_isatty = False
        pass
        self_output = None
    os.dup2(tmp.fileno(), stdout_fd)
self_save_signals = SaveSignals()
self_save_signals.save()
pass
if verbose and self_output is not None:
    sys.stderr.flush()
    sys.stdout.flush()
    print(file=self_output, flush=True)
self_stdscr = curses.initscr()
if self_isatty:
    curses.savetty()
    pass
    pass
self_stdscr.erase()
y, x = curses.getsyx()

assert isinstance(y, int)

assert isinstance(x, int)
curses.setsyx(4, 5)

assert curses.getsyx() == (4, 5)
print("TestCurses::test_getsyx: ok")
