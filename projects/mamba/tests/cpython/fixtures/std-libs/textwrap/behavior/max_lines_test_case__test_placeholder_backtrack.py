# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "max_lines_test_case__test_placeholder_backtrack"
# subject = "cpython.test_textwrap.MaxLinesTestCase.test_placeholder_backtrack"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_textwrap.py::MaxLinesTestCase::test_placeholder_backtrack
"""Auto-ported test: MaxLinesTestCase::test_placeholder_backtrack (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
text = "Hello there, how are you this fine day?  I'm glad to hear it!"

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
text = 'Good grief Python features are advancing quickly!'
check_wrap(text, 12, ['Good grief', 'Python*****'], max_lines=3, placeholder='*****')
print("MaxLinesTestCase::test_placeholder_backtrack: ok")
