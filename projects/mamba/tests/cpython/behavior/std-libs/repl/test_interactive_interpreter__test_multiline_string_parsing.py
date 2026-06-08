# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "repl"
# dimension = "behavior"
# case = "test_interactive_interpreter__test_multiline_string_parsing"
# subject = "cpython.test_repl.TestInteractiveInterpreter.test_multiline_string_parsing"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_repl.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_repl.py::TestInteractiveInterpreter::test_multiline_string_parsing
"""Auto-ported test: TestInteractiveInterpreter::test_multiline_string_parsing (CPython 3.12 oracle)."""


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
user_input = '        x = """<?xml version="1.0" encoding="iso-8859-1"?>\n        <test>\n            <Users>\n                <fun25>\n                    <limits>\n                        <total>0KiB</total>\n                        <kbps>0</kbps>\n                        <rps>1.3</rps>\n                        <connections>0</connections>\n                    </limits>\n                    <usages>\n                        <total>16738211KiB</total>\n                        <kbps>237.15</kbps>\n                        <rps>1.3</rps>\n                        <connections>0</connections>\n                    </usages>\n                    <time_to_refresh>never</time_to_refresh>\n                    <limit_exceeded_URL>none</limit_exceeded_URL>\n                </fun25>\n            </Users>\n        </test>"""\n        '
user_input = dedent(user_input)
p = spawn_repl()
p.stdin.write(user_input)
output = kill_python(p)

assert p.returncode == 0
print("TestInteractiveInterpreter::test_multiline_string_parsing: ok")
