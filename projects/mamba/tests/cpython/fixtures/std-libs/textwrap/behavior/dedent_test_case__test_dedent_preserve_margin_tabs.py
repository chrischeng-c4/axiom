# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_test_case__test_dedent_preserve_margin_tabs"
# subject = "cpython.test_textwrap.DedentTestCase.test_dedent_preserve_margin_tabs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_textwrap.py::DedentTestCase::test_dedent_preserve_margin_tabs
"""Auto-ported test: DedentTestCase::test_dedent_preserve_margin_tabs (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def assertUnchanged(text):
    """assert that dedent() has no effect on 'text'"""

    assert text == dedent(text)
text = '  hello there\n\thow are you?'
assertUnchanged(text)
text = '        hello there\n\thow are you?'
assertUnchanged(text)
text = '\thello there\n\thow are you?'
expect = 'hello there\nhow are you?'

assert expect == dedent(text)
text = '  \thello there\n  \thow are you?'

assert expect == dedent(text)
text = '  \t  hello there\n  \t  how are you?'

assert expect == dedent(text)
text = '  \thello there\n  \t  how are you?'
expect = 'hello there\n  how are you?'

assert expect == dedent(text)
text = "  \thello there\n   \thow are you?\n \tI'm fine, thanks"
expect = " \thello there\n  \thow are you?\n\tI'm fine, thanks"

assert expect == dedent(text)
print("DedentTestCase::test_dedent_preserve_margin_tabs: ok")
