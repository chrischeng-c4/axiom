# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_break_on_hyphens"
# subject = "cpython.test_textwrap.WrapTestCase.test_break_on_hyphens"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_break_on_hyphens
"""Auto-ported test: WrapTestCase::test_break_on_hyphens (CPython 3.12 oracle)."""


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
text = 'yaba daba-doo'
check_wrap(text, 10, ['yaba daba-', 'doo'], break_on_hyphens=True)
check_wrap(text, 10, ['yaba', 'daba-doo'], break_on_hyphens=False)
print("WrapTestCase::test_break_on_hyphens: ok")
