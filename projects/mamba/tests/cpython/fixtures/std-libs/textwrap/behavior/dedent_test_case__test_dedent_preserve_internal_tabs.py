# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_test_case__test_dedent_preserve_internal_tabs"
# subject = "cpython.test_textwrap.DedentTestCase.test_dedent_preserve_internal_tabs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_textwrap.py::DedentTestCase::test_dedent_preserve_internal_tabs
"""Auto-ported test: DedentTestCase::test_dedent_preserve_internal_tabs (CPython 3.12 oracle)."""


import unittest
from textwrap import TextWrapper, wrap, fill, dedent, indent, shorten


class BaseTestCase(unittest.TestCase):
    """Parent class with utility methods for textwrap tests."""

    def show(self, textin):
        if isinstance(textin, list):
            result = []
            for i in range(len(textin)):
                result.append('  %d: %r' % (i, textin[i]))
            result = '\n'.join(result) if result else '  no lines'
        elif isinstance(textin, str):
            result = '  %s\n' % repr(textin)
        return result

    def check(self, result, expect):
        self.assertEqual(result, expect, 'expected:\n%s\nbut got:\n%s' % (self.show(expect), self.show(result)))

    def check_wrap(self, text, width, expect, **kwargs):
        result = wrap(text, width, **kwargs)
        self.check(result, expect)

    def check_split(self, text, expect):
        result = self.wrapper._split(text)
        self.assertEqual(result, expect, '\nexpected %r\nbut got  %r' % (expect, result))


# --- test body ---
text = '  hello\tthere\n  how are\tyou?'
expect = 'hello\tthere\nhow are\tyou?'

assert expect == dedent(text)

assert expect == dedent(expect)
print("DedentTestCase::test_dedent_preserve_internal_tabs: ok")
