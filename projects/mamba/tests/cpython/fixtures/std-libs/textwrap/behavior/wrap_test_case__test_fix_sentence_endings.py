# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "wrap_test_case__test_fix_sentence_endings"
# subject = "cpython.test_textwrap.WrapTestCase.test_fix_sentence_endings"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_textwrap.py::WrapTestCase::test_fix_sentence_endings
"""Auto-ported test: WrapTestCase::test_fix_sentence_endings (CPython 3.12 oracle)."""


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
wrapper = TextWrapper(60, fix_sentence_endings=True)
text = 'A short line. Note the single space.'
expect = ['A short line.  Note the single space.']
check(wrapper.wrap(text), expect)
text = 'Well, Doctor? What do you think?'
expect = ['Well, Doctor?  What do you think?']
check(wrapper.wrap(text), expect)
text = 'Well, Doctor?\nWhat do you think?'
check(wrapper.wrap(text), expect)
text = 'I say, chaps! Anyone for "tennis?"\nHmmph!'
expect = ['I say, chaps!  Anyone for "tennis?"  Hmmph!']
check(wrapper.wrap(text), expect)
wrapper.width = 20
expect = ['I say, chaps!', 'Anyone for "tennis?"', 'Hmmph!']
check(wrapper.wrap(text), expect)
text = 'And she said, "Go to hell!"\nCan you believe that?'
expect = ['And she said, "Go to', 'hell!"  Can you', 'believe that?']
check(wrapper.wrap(text), expect)
wrapper.width = 60
expect = ['And she said, "Go to hell!"  Can you believe that?']
check(wrapper.wrap(text), expect)
text = 'File stdio.h is nice.'
expect = ['File stdio.h is nice.']
check(wrapper.wrap(text), expect)
print("WrapTestCase::test_fix_sentence_endings: ok")
