# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "shorten_test_case__test_simple"
# subject = "cpython.test_textwrap.ShortenTestCase.test_simple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::ShortenTestCase::test_simple
"""Auto-ported test: ShortenTestCase::test_simple (CPython 3.12 oracle)."""


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
text = "Hello there, how are you this fine day? I'm glad to hear it!"
check_shorten(text, 18, 'Hello there, [...]')
check_shorten(text, len(text), text)
check_shorten(text, len(text) - 1, "Hello there, how are you this fine day? I'm glad to [...]")
print("ShortenTestCase::test_simple: ok")
