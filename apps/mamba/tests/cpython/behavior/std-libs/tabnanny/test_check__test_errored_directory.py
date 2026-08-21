# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tabnanny"
# dimension = "behavior"
# case = "test_check__test_errored_directory"
# subject = "cpython.test_tabnanny.TestCheck.test_errored_directory"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tabnanny.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_tabnanny.py::TestCheck::test_errored_directory
"""Auto-ported test: TestCheck::test_errored_directory (CPython 3.12 oracle)."""


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
def verify_tabnanny_check(dir_or_file, out='', err=''):
    """Common verification for tabnanny.check().

        Use this method to assert expected values of `stdout` and `stderr` after
        running tabnanny.check() on given `dir` or `file` path. Because
        tabnanny.check() captures exceptions and writes to `stdout` and
        `stderr`, asserting standard outputs is the only way.
        """
    with captured_stdout() as stdout, captured_stderr() as stderr:
        tabnanny.check(dir_or_file)

    assert stdout.getvalue() == out

    assert stderr.getvalue() == err
pass
tabnanny.verbose = 0
'Directory containing wrongly indented python source code files.'
with tempfile.TemporaryDirectory() as tmp_dir:
    error_file = TemporaryPyFile(SOURCE_CODES['wrong_indented'], directory=tmp_dir)
    code_file = TemporaryPyFile(SOURCE_CODES['error_free'], directory=tmp_dir)
    with error_file as e_file, code_file as c_file:
        err = 'unindent does not match any outer indentation level (<tokenize>, line 3)\n'
        err = f'{e_file!r}: Indentation Error: {err}'
        try:
            verify_tabnanny_check(tmp_dir, err=err)
            raise AssertionError('expected SystemExit')
        except SystemExit:
            pass
print("TestCheck::test_errored_directory: ok")
