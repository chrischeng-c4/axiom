# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "behavior"
# case = "test_history_manipulation__test_history_updates"
# subject = "cpython.test_readline.TestHistoryManipulation.testHistoryUpdates"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_readline.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_readline.py::TestHistoryManipulation::testHistoryUpdates
"""Auto-ported test: TestHistoryManipulation::testHistoryUpdates (CPython 3.12 oracle)."""


import locale
import os
import sys
import tempfile
import textwrap
import unittest
from test.support import verbose
from test.support.import_helper import import_module
from test.support.os_helper import unlink, temp_dir, TESTFN
from test.support.pty_helper import run_pty
from test.support.script_helper import assert_python_ok
from test.support.threading_helper import requires_working_threading


'\nVery minimal unittests for parts of the readline module.\n'

readline = import_module('readline')

if hasattr(readline, '_READLINE_LIBRARY_VERSION'):
    is_editline = 'EditLine wrapper' in readline._READLINE_LIBRARY_VERSION
else:
    is_editline = readline.__doc__ and 'libedit' in readline.__doc__

def setUpModule():
    if verbose:
        if hasattr(readline, '_READLINE_VERSION'):
            print(f'readline version: {readline._READLINE_VERSION:#x}')
            print(f'readline runtime version: {readline._READLINE_RUNTIME_VERSION:#x}')
        if hasattr(readline, '_READLINE_LIBRARY_VERSION'):
            print(f'readline library version: {readline._READLINE_LIBRARY_VERSION!r}')
        print(f'use libedit emulation? {is_editline}')


# --- test body ---
readline.clear_history()
readline.add_history('first line')
readline.add_history('second line')

assert readline.get_history_item(0) == None

assert readline.get_history_item(1) == 'first line'

assert readline.get_history_item(2) == 'second line'
readline.replace_history_item(0, 'replaced line')

assert readline.get_history_item(0) == None

assert readline.get_history_item(1) == 'replaced line'

assert readline.get_history_item(2) == 'second line'

assert readline.get_current_history_length() == 2
readline.remove_history_item(0)

assert readline.get_history_item(0) == None

assert readline.get_history_item(1) == 'second line'

assert readline.get_current_history_length() == 1
print("TestHistoryManipulation::testHistoryUpdates: ok")
