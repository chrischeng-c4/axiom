# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_test_cases__test_initial_indent"
# subject = "cpython.test_textwrap.IndentTestCases.test_initial_indent"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_textwrap.py::IndentTestCases::test_initial_indent
"""Auto-ported test: IndentTestCases::test_initial_indent (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_split(text, expect):
    result = self_wrapper._split(text)

    assert result == expect

def check_wrap(text, width, expect, **kwargs):
    result = wrap(text, width, **kwargs)
    check(result, expect)

def show(textin):
    if isinstance(textin, list):
        result = []
        for i in range(len(textin)):
            result.append('  %d: %r' % (i, textin[i]))
        result = '\n'.join(result) if result else '  no lines'
    elif isinstance(textin, str):
        result = '  %s\n' % repr(textin)
    return result
self_text = 'This paragraph will be filled, first without any indentation,\nand then with some (including a hanging indent).'
expect = ['     This paragraph will be filled,', 'first without any indentation, and then', 'with some (including a hanging indent).']
result = wrap(self_text, 40, initial_indent='     ')
check(result, expect)
expect = '\n'.join(expect)
result = fill(self_text, 40, initial_indent='     ')
check(result, expect)
print("IndentTestCases::test_initial_indent: ok")
