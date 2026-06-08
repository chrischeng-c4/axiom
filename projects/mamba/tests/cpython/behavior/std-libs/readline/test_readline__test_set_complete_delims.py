# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "readline"
# dimension = "behavior"
# case = "test_readline__test_set_complete_delims"
# subject = "cpython.test_readline.TestReadline.test_set_complete_delims"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_readline.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_readline.py::TestReadline::test_set_complete_delims
"""Auto-ported test: TestReadline::test_set_complete_delims (CPython 3.12 oracle)."""


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
auto_history_script = 'import readline\nreadline.set_auto_history({})\ninput()\nprint("History length:", readline.get_current_history_length())\n'
script = textwrap.dedent('\n            import readline\n            def complete(text, state):\n                if state == 0 and text == "$":\n                    return "$complete"\n                return None\n            if "libedit" in getattr(readline, "__doc__", ""):\n                readline.parse_and_bind(r\'bind "\\t" rl_complete\')\n            else:\n                readline.parse_and_bind(r\'"\\t": complete\')\n            readline.set_completer_delims(" \\t\\n")\n            readline.set_completer(complete)\n            print(input())\n        ')
output = run_pty(script, input=b'$\t\n')

assert b'$complete' in output
print("TestReadline::test_set_complete_delims: ok")
