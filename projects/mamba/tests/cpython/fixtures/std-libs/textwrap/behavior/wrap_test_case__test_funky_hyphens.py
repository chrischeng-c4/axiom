# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_funky_hyphens"
# subject = "cpython.test_textwrap.WrapTestCase.test_funky_hyphens"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_funky_hyphens
"""Auto-ported test: WrapTestCase::test_funky_hyphens (CPython 3.12 oracle)."""


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
check_split('what the--hey!', ['what', ' ', 'the', '--', 'hey!'])
check_split('what the--', ['what', ' ', 'the--'])
check_split('what the--.', ['what', ' ', 'the--.'])
check_split('--text--.', ['--text--.'])
check_split('--option', ['--option'])
check_split('--option-opt', ['--option-', 'opt'])
check_split('foo --option-opt bar', ['foo', ' ', '--option-', 'opt', ' ', 'bar'])
print("WrapTestCase::test_funky_hyphens: ok")
