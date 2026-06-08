# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "repl"
# dimension = "behavior"
# case = "test_interactive_mode_syntax_errors__test_interactive_syntax_error_correct_line"
# subject = "cpython.test_repl.TestInteractiveModeSyntaxErrors.test_interactive_syntax_error_correct_line"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_repl.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_repl.py::TestInteractiveModeSyntaxErrors::test_interactive_syntax_error_correct_line
"""Auto-ported test: TestInteractiveModeSyntaxErrors::test_interactive_syntax_error_correct_line (CPython 3.12 oracle)."""


import sys
import os
import unittest
import subprocess
from textwrap import dedent
from test.support import cpython_only, has_subprocess_support, SuppressCrashReport
from test.support.script_helper import kill_python


'Test the interactive interpreter.'

if not has_subprocess_support:
    raise unittest.SkipTest('test module requires subprocess')

def spawn_repl(*args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, **kw):
    """Run the Python REPL with the given arguments.

    kw is extra keyword args to pass to subprocess.Popen. Returns a Popen
    object.
    """
    stdin_fname = os.path.join(os.path.dirname(sys.executable), '<stdin>')
    cmd_line = [stdin_fname, '-E', '-i']
    cmd_line.extend(args)
    env = kw.setdefault('env', dict(os.environ))
    env['TERM'] = 'vt100'
    return subprocess.Popen(cmd_line, executable=sys.executable, text=True, stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, **kw)

def run_on_interactive_mode(source):
    """Spawn a new Python interpreter, pass the given
    input source code from the stdin and return the
    result back. If the interpreter exits non-zero, it
    raises a ValueError."""
    process = spawn_repl()
    process.stdin.write(source)
    output = kill_python(process)
    if process.returncode != 0:
        raise ValueError("Process didn't exit properly.")
    return output


# --- test body ---
output = run_on_interactive_mode(dedent('        def f():\n            print(0)\n            return yield 42\n        '))
traceback_lines = output.splitlines()[-4:-1]
expected_lines = ['    return yield 42', '           ^^^^^', 'SyntaxError: invalid syntax']

assert traceback_lines == expected_lines
print("TestInteractiveModeSyntaxErrors::test_interactive_syntax_error_correct_line: ok")
