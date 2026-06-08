# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "long_word_test_case__test_break_long"
# subject = "cpython.test_textwrap.LongWordTestCase.test_break_long"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::LongWordTestCase::test_break_long
"""Auto-ported test: LongWordTestCase::test_break_long (CPython 3.12 oracle)."""


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
self_wrapper = TextWrapper()
self_text = 'Did you say "supercalifragilisticexpialidocious?"\nHow *do* you spell that odd word, anyways?\n'
check_wrap(self_text, 30, ['Did you say "supercalifragilis', 'ticexpialidocious?" How *do*', 'you spell that odd word,', 'anyways?'])
check_wrap(self_text, 50, ['Did you say "supercalifragilisticexpialidocious?"', 'How *do* you spell that odd word, anyways?'])
check_wrap('-' * 10 + 'hello', 10, ['----------', '               h', '               e', '               l', '               l', '               o'], subsequent_indent=' ' * 15)
check_wrap(self_text, 12, ['Did you say ', '"supercalifr', 'agilisticexp', 'ialidocious?', '" How *do*', 'you spell', 'that odd', 'word,', 'anyways?'])
print("LongWordTestCase::test_break_long: ok")
