# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_whitespace"
# subject = "cpython.test_textwrap.WrapTestCase.test_whitespace"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_whitespace
"""Auto-ported test: WrapTestCase::test_whitespace (CPython 3.12 oracle)."""


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
text = 'This is a paragraph that already has\nline breaks.  But some of its lines are much longer than the others,\nso it needs to be wrapped.\nSome lines are \ttabbed too.\nWhat a mess!\n'
expect = ['This is a paragraph that already has line', 'breaks.  But some of its lines are much', 'longer than the others, so it needs to be', 'wrapped.  Some lines are  tabbed too.  What a', 'mess!']
wrapper = TextWrapper(45, fix_sentence_endings=True)
result = wrapper.wrap(text)
check(result, expect)
result = wrapper.fill(text)
check(result, '\n'.join(expect))
text = '\tTest\tdefault\t\ttabsize.'
expect = ['        Test    default         tabsize.']
check_wrap(text, 80, expect)
text = '\tTest\tcustom\t\ttabsize.'
expect = ['    Test    custom      tabsize.']
check_wrap(text, 80, expect, tabsize=4)
print("WrapTestCase::test_whitespace: ok")
