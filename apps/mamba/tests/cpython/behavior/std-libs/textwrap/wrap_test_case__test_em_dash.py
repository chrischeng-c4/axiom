# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_em_dash"
# subject = "cpython.test_textwrap.WrapTestCase.test_em_dash"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_em_dash
"""Auto-ported test: WrapTestCase::test_em_dash (CPython 3.12 oracle)."""


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
text = 'Em-dashes should be written -- thus.'
check_wrap(text, 25, ['Em-dashes should be', 'written -- thus.'])
check_wrap(text, 29, ['Em-dashes should be written', '-- thus.'])
expect = ['Em-dashes should be written --', 'thus.']
check_wrap(text, 30, expect)
check_wrap(text, 35, expect)
check_wrap(text, 36, ['Em-dashes should be written -- thus.'])
text = 'You can also do--this or even---this.'
expect = ['You can also do', '--this or even', '---this.']
check_wrap(text, 15, expect)
check_wrap(text, 16, expect)
expect = ['You can also do--', 'this or even---', 'this.']
check_wrap(text, 17, expect)
check_wrap(text, 19, expect)
expect = ['You can also do--this or even', '---this.']
check_wrap(text, 29, expect)
check_wrap(text, 31, expect)
expect = ['You can also do--this or even---', 'this.']
check_wrap(text, 32, expect)
check_wrap(text, 35, expect)
text = "Here's an -- em-dash and--here's another---and another!"
expect = ["Here's", ' ', 'an', ' ', '--', ' ', 'em-', 'dash', ' ', 'and', '--', "here's", ' ', 'another', '---', 'and', ' ', 'another!']
check_split(text, expect)
text = 'and then--bam!--he was gone'
expect = ['and', ' ', 'then', '--', 'bam!', '--', 'he', ' ', 'was', ' ', 'gone']
check_split(text, expect)
print("WrapTestCase::test_em_dash: ok")
