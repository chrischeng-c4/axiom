# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeop"
# dimension = "behavior"
# case = "codeop_tests__test_syntax_errors"
# subject = "cpython.test_codeop.CodeopTests.test_syntax_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_codeop.py::CodeopTests::test_syntax_errors
"""Auto-ported test: CodeopTests::test_syntax_errors (CPython 3.12 oracle)."""


import unittest
import warnings
from test.support import warnings_helper
from textwrap import dedent
from codeop import compile_command, PyCF_DONT_IMPLY_DEDENT


'\n   Test cases for codeop.py\n   Nick Mathewson\n'


# --- test body ---
def assertIncomplete(str, symbol='single'):
    """succeed iff str is the start of a valid piece of code"""

    assert compile_command(str, symbol=symbol) == None

def assertInvalid(str, symbol='single', is_syntax=1):
    """succeed iff str is the start of an invalid piece of code"""
    try:
        compile_command(str, symbol=symbol)

        raise AssertionError('No exception raised for invalid code')
    except SyntaxError:

        assert is_syntax
    except OverflowError:

        assert not is_syntax

def assertSyntaxErrorMatches(code, message):
    try:
        compile_command(code, symbol='exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError as _aR_e:
        import re as _re_aR
        assert _re_aR.search(message, str(_aR_e))

def assertValid(str, symbol='single'):
    """succeed iff str is a valid piece of code"""
    expected = compile(str, '<input>', symbol, PyCF_DONT_IMPLY_DEDENT)

    assert compile_command(str, '<input>', symbol) == expected
assertSyntaxErrorMatches(dedent('                def foo(x,x):\n                   pass\n            '), "duplicate argument 'x' in function definition")
print("CodeopTests::test_syntax_errors: ok")
