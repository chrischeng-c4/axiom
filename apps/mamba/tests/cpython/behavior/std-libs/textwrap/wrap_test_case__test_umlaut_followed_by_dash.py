# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_umlaut_followed_by_dash"
# subject = "cpython.test_textwrap.WrapTestCase.test_umlaut_followed_by_dash"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_umlaut_followed_by_dash
"""Auto-ported test: WrapTestCase::test_umlaut_followed_by_dash (CPython 3.12 oracle)."""


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
self_wrapper = TextWrapper(width=45)
text = 'aa ää-ää'
check_wrap(text, 7, ['aa ää-', 'ää'])
print("WrapTestCase::test_umlaut_followed_by_dash: ok")
