# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_unix_options"
# subject = "cpython.test_textwrap.WrapTestCase.test_unix_options"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_unix_options
"""Auto-ported test: WrapTestCase::test_unix_options (CPython 3.12 oracle)."""


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
text = 'You should use the -n option, or --dry-run in its long form.'
check_wrap(text, 20, ['You should use the', '-n option, or --dry-', 'run in its long', 'form.'])
check_wrap(text, 21, ['You should use the -n', 'option, or --dry-run', 'in its long form.'])
expect = ['You should use the -n option, or', '--dry-run in its long form.']
check_wrap(text, 32, expect)
check_wrap(text, 34, expect)
check_wrap(text, 35, expect)
check_wrap(text, 38, expect)
expect = ['You should use the -n option, or --dry-', 'run in its long form.']
check_wrap(text, 39, expect)
check_wrap(text, 41, expect)
expect = ['You should use the -n option, or --dry-run', 'in its long form.']
check_wrap(text, 42, expect)
text = 'the -n option, or --dry-run or --dryrun'
expect = ['the', ' ', '-n', ' ', 'option,', ' ', 'or', ' ', '--dry-', 'run', ' ', 'or', ' ', '--dryrun']
check_split(text, expect)
print("WrapTestCase::test_unix_options: ok")
