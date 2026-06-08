# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "shorten_test_case__test_whitespace"
# subject = "cpython.test_textwrap.ShortenTestCase.test_whitespace"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_textwrap.py::ShortenTestCase::test_whitespace
"""Auto-ported test: ShortenTestCase::test_whitespace (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def check(result, expect):

    assert result == expect

def check_shorten(text, width, expect, **kwargs):
    result = shorten(text, width, **kwargs)
    check(result, expect)

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
text = '\n            This is a  paragraph that  already has\n            line breaks and \t tabs too.'
check_shorten(text, 62, 'This is a paragraph that already has line breaks and tabs too.')
check_shorten(text, 61, 'This is a paragraph that already has line breaks and [...]')
check_shorten('hello      world!  ', 12, 'hello world!')
check_shorten('hello      world!  ', 11, 'hello [...]')
check_shorten('hello      world!  ', 10, '[...]')
print("ShortenTestCase::test_whitespace: ok")
