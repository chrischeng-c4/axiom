# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "max_lines_test_case__test_simple"
# subject = "cpython.test_textwrap.MaxLinesTestCase.test_simple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::MaxLinesTestCase::test_simple
"""Auto-ported test: MaxLinesTestCase::test_simple (CPython 3.12 oracle)."""


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
check_wrap(text, 12, ['Hello [...]'], max_lines=0)
check_wrap(text, 12, ['Hello [...]'], max_lines=1)
check_wrap(text, 12, ['Hello there,', 'how [...]'], max_lines=2)
check_wrap(text, 13, ['Hello there,', 'how are [...]'], max_lines=2)
check_wrap(text, 80, [text], max_lines=1)
check_wrap(text, 12, ['Hello there,', 'how are you', 'this fine', "day?  I'm", 'glad to hear', 'it!'], max_lines=6)
print("MaxLinesTestCase::test_simple: ok")
