# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "textwrap"
# dimension = "behavior"
# case = "indent_test_case__test_indent_all_lines"
# subject = "cpython.test_textwrap.IndentTestCase.test_indent_all_lines"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_textwrap.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_textwrap.py::IndentTestCase::test_indent_all_lines
"""Auto-ported test: IndentTestCase::test_indent_all_lines (CPython 3.12 oracle)."""


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
ROUNDTRIP_CASES = ('Hi.\nThis is a test.\nTesting.', 'Hi.\nThis is a test.\n\nTesting.', '\nHi.\nThis is a test.\nTesting.\n')
CASES = ROUNDTRIP_CASES + ('Hi.\r\nThis is a test.\r\nTesting.\r\n', '\nHi.\r\nThis is a test.\n\r\nTesting.\r\n\n')
prefix = '  '
expected = ('  Hi.\n  This is a test.\n  Testing.', '  Hi.\n  This is a test.\n  \n  Testing.', '  \n  Hi.\n  This is a test.\n  Testing.\n', '  Hi.\r\n  This is a test.\r\n  Testing.\r\n', '  \n  Hi.\r\n  This is a test.\n  \r\n  Testing.\r\n  \n')
predicate = lambda line: True
for text, expect in zip(CASES, expected):

    assert indent(text, prefix, predicate) == expect
print("IndentTestCase::test_indent_all_lines: ok")
