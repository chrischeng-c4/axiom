# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_test_case__test_dedent_nomargin"
# subject = "cpython.test_textwrap.DedentTestCase.test_dedent_nomargin"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_textwrap.py::DedentTestCase::test_dedent_nomargin
"""Auto-ported test: DedentTestCase::test_dedent_nomargin (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


# --- test body ---
def assertUnchanged(text):
    """assert that dedent() has no effect on 'text'"""

    assert text == dedent(text)
text = "Hello there.\nHow are you?\nOh good, I'm glad."
assertUnchanged(text)
text = 'Hello there.\n\nBoo!'
assertUnchanged(text)
text = 'Hello there.\n  This is indented.'
assertUnchanged(text)
text = 'Hello there.\n\n  Boo!\n'
assertUnchanged(text)
print("DedentTestCase::test_dedent_nomargin: ok")
