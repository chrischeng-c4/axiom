# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeop"
# dimension = "behavior"
# case = "codeop_tests__test_incomplete"
# subject = "cpython.test_codeop.CodeopTests.test_incomplete"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codeop.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_codeop.py::CodeopTests::test_incomplete
"""Auto-ported test: CodeopTests::test_incomplete (CPython 3.12 oracle)."""


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
ai = assertIncomplete
ai('(a **')
ai('(a,b,')
ai('(a,b,(')
ai('(a,b,(')
ai('a = (')
ai('a = {')
ai('b + {')
ai('print([1,\n2,')
ai('print({1:1,\n2:3,')
ai('print((1,\n2,')
ai('if 9==3:\n   pass\nelse:')
ai('if 9==3:\n   pass\nelse:\n')
ai('if 9==3:\n   pass\nelse:\n   pass')
ai('if 1:')
ai('if 1:\n')
ai('if 1:\n pass\n if 1:\n  pass\n else:')
ai('if 1:\n pass\n if 1:\n  pass\n else:\n')
ai('if 1:\n pass\n if 1:\n  pass\n else:\n  pass')
ai('def x():')
ai('def x():\n')
ai('def x():\n\n')
ai('def x():\n  pass')
ai('def x():\n  pass\n ')
ai('def x():\n  pass\n  ')
ai('\n\ndef x():\n  pass')
ai('a = 9+ \\')
ai("a = 'a\\")
ai("a = '''xy")
ai('', 'eval')
ai('\n', 'eval')
ai('(', 'eval')
ai('(9+', 'eval')
ai('9+ \\', 'eval')
ai('lambda z: \\', 'eval')
ai('if True:\n if True:\n  if True:   \n')
ai('@a(')
ai('@a(b')
ai('@a(b,')
ai('@a(b,c')
ai('@a(b,c,')
ai('from a import (')
ai('from a import (b')
ai('from a import (b,')
ai('from a import (b,c')
ai('from a import (b,c,')
ai('[')
ai('[a')
ai('[a,')
ai('[a,b')
ai('[a,b,')
ai('{')
ai('{a')
ai('{a:')
ai('{a:b')
ai('{a:b,')
ai('{a:b,c')
ai('{a:b,c:')
ai('{a:b,c:d')
ai('{a:b,c:d,')
ai('a(')
ai('a(b')
ai('a(b,')
ai('a(b,c')
ai('a(b,c,')
ai('a[')
ai('a[b')
ai('a[b,')
ai('a[b:')
ai('a[b:c')
ai('a[b:c:')
ai('a[b:c:d')
ai('def a(')
ai('def a(b')
ai('def a(b,')
ai('def a(b,c')
ai('def a(b,c,')
ai('(')
ai('(a')
ai('(a,')
ai('(a,b')
ai('(a,b,')
ai('if a:\n pass\nelif b:')
ai('if a:\n pass\nelif b:\n pass\nelse:')
ai('while a:')
ai('while a:\n pass\nelse:')
ai('for a in b:')
ai('for a in b:\n pass\nelse:')
ai('try:')
ai('try:\n pass\nexcept:')
ai('try:\n pass\nfinally:')
ai('try:\n pass\nexcept:\n pass\nfinally:')
ai('with a:')
ai('with a as b:')
ai('class a:')
ai('class a(')
ai('class a(b')
ai('class a(b,')
ai('class a():')
ai('[x for')
ai('[x for x in')
ai('[x for x in (')
ai('(x for')
ai('(x for x in')
ai('(x for x in (')
ai('a = f"""')
ai('a = \\')
print("CodeopTests::test_incomplete: ok")
