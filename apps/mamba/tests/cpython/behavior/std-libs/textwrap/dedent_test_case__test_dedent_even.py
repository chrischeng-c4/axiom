# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "dedent_test_case__test_dedent_even"
# subject = "cpython.test_textwrap.DedentTestCase.test_dedent_even"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_textwrap.py::DedentTestCase::test_dedent_even
"""Auto-ported test: DedentTestCase::test_dedent_even (CPython 3.12 oracle)."""


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
text = '  Hello there.\n  How are ya?\n  Oh good.'
expect = 'Hello there.\nHow are ya?\nOh good.'

assert expect == dedent(text)
text = '  Hello there.\n\n  How are ya?\n  Oh good.\n'
expect = 'Hello there.\n\nHow are ya?\nOh good.\n'

assert expect == dedent(text)
text = '  Hello there.\n  \n  How are ya?\n  Oh good.\n'
expect = 'Hello there.\n\nHow are ya?\nOh good.\n'

assert expect == dedent(text)
print("DedentTestCase::test_dedent_even: ok")
