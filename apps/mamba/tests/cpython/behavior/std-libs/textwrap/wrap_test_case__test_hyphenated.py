# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_hyphenated"
# subject = "cpython.test_textwrap.WrapTestCase.test_hyphenated"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_hyphenated
"""Auto-ported test: WrapTestCase::test_hyphenated (CPython 3.12 oracle)."""


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
text = "this-is-a-useful-feature-for-reformatting-posts-from-tim-peters'ly"
check_wrap(text, 40, ['this-is-a-useful-feature-for-', "reformatting-posts-from-tim-peters'ly"])
check_wrap(text, 41, ['this-is-a-useful-feature-for-', "reformatting-posts-from-tim-peters'ly"])
check_wrap(text, 42, ['this-is-a-useful-feature-for-reformatting-', "posts-from-tim-peters'ly"])
expect = "this-|is-|a-|useful-|feature-|for-|reformatting-|posts-|from-|tim-|peters'ly".split('|')
check_wrap(text, 1, expect, break_long_words=False)
check_split(text, expect)
check_split('e-mail', ['e-mail'])
check_split('Jelly-O', ['Jelly-O'])
check_split('half-a-crown', 'half-|a-|crown'.split('|'))
print("WrapTestCase::test_hyphenated: ok")
