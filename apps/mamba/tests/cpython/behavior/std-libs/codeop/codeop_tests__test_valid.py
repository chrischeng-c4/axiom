# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeop"
# dimension = "behavior"
# case = "codeop_tests__test_valid"
# subject = "cpython.test_codeop.CodeopTests.test_valid"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_codeop.py::CodeopTests::test_valid
"""Auto-ported test: CodeopTests::test_valid (CPython 3.12 oracle)."""


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
av = assertValid

assert compile_command('') == compile('pass', '<input>', 'single', PyCF_DONT_IMPLY_DEDENT)

assert compile_command('\n') == compile('pass', '<input>', 'single', PyCF_DONT_IMPLY_DEDENT)
av('a = 1')
av('\na = 1')
av('a = 1\n')
av('a = 1\n\n')
av('\n\na = 1\n\n')
av('def x():\n  pass\n')
av('if 1:\n pass\n')
av('\n\nif 1: pass\n')
av('\n\nif 1: pass\n\n')
av('def x():\n\n pass\n')
av('def x():\n  pass\n  \n')
av('def x():\n  pass\n \n')
av('pass\n')
av('3**3\n')
av('if 9==3:\n   pass\nelse:\n   pass\n')
av('if 1:\n pass\n if 1:\n  pass\n else:\n  pass\n')
av('#a\n#b\na = 3\n')
av('#a\n\n   \na=3\n')
av('a=3\n\n')
av('a = 9+ \\\n3')
av('3**3', 'eval')
av('(lambda z: \n z**3)', 'eval')
av('9+ \\\n3', 'eval')
av('9+ \\\n3\n', 'eval')
av('\n\na**3', 'eval')
av('\n \na**3', 'eval')
av('#a\n#b\na**3', 'eval')
av('\n\na = 1\n\n')
av('\n\nif 1: a=1\n\n')
av('if 1:\n pass\n if 1:\n  pass\n else:\n  pass\n')
av('#a\n\n   \na=3\n\n')
av('\n\na**3', 'eval')
av('\n \na**3', 'eval')
av('#a\n#b\na**3', 'eval')
av('def f():\n try: pass\n finally: [x for x in (1,2)]\n')
av('def f():\n pass\n#foo\n')
av('@a.b.c\ndef f():\n pass\n')
print("CodeopTests::test_valid: ok")
