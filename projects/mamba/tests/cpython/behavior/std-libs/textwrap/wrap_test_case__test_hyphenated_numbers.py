# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_hyphenated_numbers"
# subject = "cpython.test_textwrap.WrapTestCase.test_hyphenated_numbers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_hyphenated_numbers
"""Auto-ported test: WrapTestCase::test_hyphenated_numbers (CPython 3.12 oracle)."""


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
text = 'Python 1.0.0 was released on 1994-01-26.  Python 1.0.1 was\nreleased on 1994-02-15.'
check_wrap(text, 30, ['Python 1.0.0 was released on', '1994-01-26.  Python 1.0.1 was', 'released on 1994-02-15.'])
check_wrap(text, 40, ['Python 1.0.0 was released on 1994-01-26.', 'Python 1.0.1 was released on 1994-02-15.'])
check_wrap(text, 1, text.split(), break_long_words=False)
text = 'I do all my shopping at 7-11.'
check_wrap(text, 25, ['I do all my shopping at', '7-11.'])
check_wrap(text, 27, ['I do all my shopping at', '7-11.'])
check_wrap(text, 29, ['I do all my shopping at 7-11.'])
check_wrap(text, 1, text.split(), break_long_words=False)
print("WrapTestCase::test_hyphenated_numbers: ok")
