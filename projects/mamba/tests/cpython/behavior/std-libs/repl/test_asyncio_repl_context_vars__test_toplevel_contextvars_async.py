# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "repl"
# dimension = "behavior"
# case = "test_asyncio_repl_context_vars__test_toplevel_contextvars_async"
# subject = "cpython.test_repl.TestAsyncioREPLContextVars.test_toplevel_contextvars_async"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_repl.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_repl.py::TestAsyncioREPLContextVars::test_toplevel_contextvars_async
"""Auto-ported test: TestAsyncioREPLContextVars::test_toplevel_contextvars_async (CPython 3.12 oracle)."""


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
user_input = dedent("        from contextvars import ContextVar\n        var = ContextVar('var', default='failed')\n        ")
p = spawn_repl('-m', 'asyncio')
p.stdin.write(user_input + '\n')
user_input2 = "async def set_var(): var.set('ok')\n"
p.stdin.write(user_input2 + '\n')
user_input3 = 'await set_var()\n'
p.stdin.write(user_input3 + '\n')
user_input4 = "print(f'toplevel contextvar test: {var.get()}')\n"
p.stdin.write(user_input4 + '\n')
output = kill_python(p)

assert p.returncode == 0
expected = 'toplevel contextvar test: ok'

assert expected in output
print("TestAsyncioREPLContextVars::test_toplevel_contextvars_async: ok")
