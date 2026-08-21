# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "behavior"
# case = "test_cgitb__test_syshook_no_logdir_default_format"
# subject = "cpython.test_cgitb.TestCgitb.test_syshook_no_logdir_default_format"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cgitb.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cgitb.py::TestCgitb::test_syshook_no_logdir_default_format
"""Auto-ported test: TestCgitb::test_syshook_no_logdir_default_format (CPython 3.12 oracle)."""


from test.support.os_helper import temp_dir
from test.support.script_helper import assert_python_failure
from test.support.warnings_helper import import_deprecated
import unittest
import sys


cgitb = import_deprecated('cgitb')


# --- test body ---
with temp_dir() as tracedir:
    rc, out, err = assert_python_failure('-c', 'import cgitb; cgitb.enable(logdir=%s); raise ValueError("Hello World")' % repr(tracedir), PYTHONIOENCODING='utf-8')
out = out.decode()

assert 'ValueError' in out

assert 'Hello World' in out

assert '<strong>&lt;module&gt;</strong>' in out

assert '<p>' in out

assert '</p>' in out
print("TestCgitb::test_syshook_no_logdir_default_format: ok")
