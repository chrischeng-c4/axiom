# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "max_lines_test_case__test_placeholder"
# subject = "cpython.test_textwrap.MaxLinesTestCase.test_placeholder"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_textwrap.py::MaxLinesTestCase::test_placeholder
"""Auto-ported test: MaxLinesTestCase::test_placeholder (CPython 3.12 oracle)."""


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
check_wrap(text, 12, ['Hello...'], max_lines=1, placeholder='...')
check_wrap(text, 12, ['Hello there,', 'how are...'], max_lines=2, placeholder='...')
try:
    wrap(text, 16, initial_indent='    ', max_lines=1, placeholder=' [truncated]...')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    wrap(text, 16, subsequent_indent='    ', max_lines=2, placeholder=' [truncated]...')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
check_wrap(text, 16, ['    Hello there,', '  [truncated]...'], max_lines=2, initial_indent='    ', subsequent_indent='  ', placeholder=' [truncated]...')
check_wrap(text, 16, ['  [truncated]...'], max_lines=1, initial_indent='  ', subsequent_indent='    ', placeholder=' [truncated]...')
check_wrap(text, 80, [text], placeholder='.' * 1000)
print("MaxLinesTestCase::test_placeholder: ok")
