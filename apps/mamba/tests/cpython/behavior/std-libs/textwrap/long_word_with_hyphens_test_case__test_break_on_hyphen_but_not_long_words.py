# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "long_word_with_hyphens_test_case__test_break_on_hyphen_but_not_long_words"
# subject = "cpython.test_textwrap.LongWordWithHyphensTestCase.test_break_on_hyphen_but_not_long_words"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::LongWordWithHyphensTestCase::test_break_on_hyphen_but_not_long_words
"""Auto-ported test: LongWordWithHyphensTestCase::test_break_on_hyphen_but_not_long_words (CPython 3.12 oracle)."""


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
self_text1 = 'We used enyzme 2-succinyl-6-hydroxy-2,4-cyclohexadiene-1-carboxylate synthase.\n'
self_text2 = '1234567890-1234567890--this_is_a_very_long_option_indeed-good-bye"\n'
expected = ['We used enyzme', '2-succinyl-6-hydroxy-2,4-cyclohexadiene-1-carboxylate', 'synthase.']
check_wrap(self_text1, 50, expected, break_long_words=False)
expected = ['We used', 'enyzme', '2-succinyl-6-hydroxy-2,4-cyclohexadiene-1-carboxylate', 'synthase.']
check_wrap(self_text1, 10, expected, break_long_words=False)
expected = ['1234567890', '-123456789', '0--this_is', '_a_very_lo', 'ng_option_', 'indeed-', 'good-bye"']
check_wrap(self_text2, 10, expected)
print("LongWordWithHyphensTestCase::test_break_on_hyphen_but_not_long_words: ok")
