# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tabnanny"
# dimension = "behavior"
# case = "test_nanny_nag__test_all_methods"
# subject = "cpython.test_tabnanny.TestNannyNag.test_all_methods"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tabnanny.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tabnanny.py::TestNannyNag::test_all_methods
"""Auto-ported test: TestNannyNag::test_all_methods (CPython 3.12 oracle)."""


from unittest import TestCase, mock
import errno
import os
import tabnanny
import tokenize
import tempfile
import textwrap
from test.support import captured_stderr, captured_stdout, script_helper, findfile
from test.support.os_helper import unlink


'Testing `tabnanny` module.\n\nGlossary:\n    * errored    : Whitespace related problems present in file.\n'

SOURCE_CODES = {'incomplete_expression': 'fruits = [\n    "Apple",\n    "Orange",\n    "Banana",\n\nprint(fruits)\n', 'wrong_indented': 'if True:\n    print("hello")\n  print("world")\nelse:\n    print("else called")\n', 'nannynag_errored': 'if True:\n \tprint("hello")\n\tprint("world")\nelse:\n    print("else called")\n', 'error_free': 'if True:\n    print("hello")\n    print("world")\nelse:\n    print("else called")\n', 'tab_space_errored_1': 'def my_func():\n\t  print("hello world")\n\t  if True:\n\t\tprint("If called")', 'tab_space_errored_2': 'def my_func():\n\t\tprint("Hello world")\n\t\tif True:\n\t        print("If called")'}

class TemporaryPyFile:
    """Create a temporary python source code file."""

    def __init__(self, source_code='', directory=None):
        self.source_code = source_code
        self.dir = directory

    def __enter__(self):
        with tempfile.NamedTemporaryFile(mode='w', dir=self.dir, suffix='.py', delete=False) as f:
            f.write(self.source_code)
        self.file_path = f.name
        return self.file_path

    def __exit__(self, exc_type, exc_value, exc_traceback):
        unlink(self.file_path)


# --- test body ---
"""Asserting behaviour of `tabnanny.NannyNag` exception."""
tests = [(tabnanny.NannyNag(0, 'foo', 'bar'), {'lineno': 0, 'msg': 'foo', 'line': 'bar'}), (tabnanny.NannyNag(5, 'testmsg', 'testline'), {'lineno': 5, 'msg': 'testmsg', 'line': 'testline'})]
for nanny, expected in tests:
    line_number = nanny.get_lineno()
    msg = nanny.get_msg()
    line = nanny.get_line()

    assert expected['lineno'] == line_number

    assert expected['msg'] == msg

    assert expected['line'] == line
print("TestNannyNag::test_all_methods: ok")
