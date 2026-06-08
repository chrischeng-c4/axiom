# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lltrace"
# dimension = "behavior"
# case = "test_ll_trace__test_lltrace_does_not_crash_on_subscript_operator"
# subject = "cpython.test_lltrace.TestLLTrace.test_lltrace_does_not_crash_on_subscript_operator"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_lltrace.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_lltrace.py::TestLLTrace::test_lltrace_does_not_crash_on_subscript_operator
"""Auto-ported test: TestLLTrace::test_lltrace_does_not_crash_on_subscript_operator (CPython 3.12 oracle)."""


import dis
import textwrap
import unittest
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok


def example():
    x = []
    for i in range(0):
        x.append(i)
    x = 'this is'
    y = 'an example'
    print(x, y)


# --- test body ---
def run_code(code):
    code = textwrap.dedent(code).strip()
    with open(os_helper.TESTFN, 'w', encoding='utf-8') as fd:
        pass
        fd.write(code)
    status, stdout, stderr = assert_python_ok(os_helper.TESTFN)

    assert stderr == b''

    assert status == 0
    result = stdout.decode('utf-8')
    if support.verbose:
        print('\n\n--- code ---')
        print(code)
        print('\n--- stdout ---')
        print(result)
        print()
    return result
stdout = run_code("\n            import code\n\n            console = code.InteractiveConsole()\n            console.push('__lltrace__ = 1')\n            console.push('a = [1, 2, 3]')\n            console.push('a[0] = 1')\n            print('unreachable if bug exists')\n        ")

assert 'unreachable if bug exists' in stdout
print("TestLLTrace::test_lltrace_does_not_crash_on_subscript_operator: ok")
