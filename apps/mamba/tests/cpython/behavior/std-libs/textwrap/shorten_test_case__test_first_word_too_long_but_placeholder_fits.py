# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "shorten_test_case__test_first_word_too_long_but_placeholder_fits"
# subject = "cpython.test_textwrap.ShortenTestCase.test_first_word_too_long_but_placeholder_fits"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::ShortenTestCase::test_first_word_too_long_but_placeholder_fits
"""Auto-ported test: ShortenTestCase::test_first_word_too_long_but_placeholder_fits (CPython 3.12 oracle)."""


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
check_shorten('Helloo', 5, '[...]')
print("ShortenTestCase::test_first_word_too_long_but_placeholder_fits: ok")
