# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line_script"
# dimension = "behavior"
# case = "cmd_line_test__test_syntaxerror_multi_line_fstring"
# subject = "cpython.test_cmd_line_script.CmdLineTest.test_syntaxerror_multi_line_fstring"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line_script.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line_script.py::CmdLineTest::test_syntaxerror_multi_line_fstring
"""Auto-ported test: CmdLineTest::test_syntaxerror_multi_line_fstring (CPython 3.12 oracle)."""


import contextlib
import importlib
import importlib.machinery
import zipimport
import unittest
import sys
import os
import os.path
import py_compile
import subprocess
import io
import textwrap
from test import support
from test.support import import_helper
from test.support import os_helper
from test.support.script_helper import make_pkg, make_script, make_zip_pkg, make_zip_script, assert_python_ok, assert_python_failure, spawn_python, kill_python


verbose = support.verbose

example_args = ['test1', 'test2', 'test3']

test_source = '# Script may be run with optimisation enabled, so don\'t rely on assert\n# statements being executed\ndef assertEqual(lhs, rhs):\n    if lhs != rhs:\n        raise AssertionError(\'%r != %r\' % (lhs, rhs))\ndef assertIdentical(lhs, rhs):\n    if lhs is not rhs:\n        raise AssertionError(\'%r is not %r\' % (lhs, rhs))\n# Check basic code execution\nresult = [\'Top level assignment\']\ndef f():\n    result.append(\'Lower level reference\')\nf()\nassertEqual(result, [\'Top level assignment\', \'Lower level reference\'])\n# Check population of magic variables\nassertEqual(__name__, \'__main__\')\nfrom importlib.machinery import BuiltinImporter\n_loader = __loader__ if __loader__ is BuiltinImporter else type(__loader__)\nprint(\'__loader__==%a\' % _loader)\nprint(\'__file__==%a\' % __file__)\nprint(\'__cached__==%a\' % __cached__)\nprint(\'__package__==%r\' % __package__)\n# Check PEP 451 details\nimport os.path\nif __package__ is not None:\n    print(\'__main__ was located through the import system\')\n    assertIdentical(__spec__.loader, __loader__)\n    expected_spec_name = os.path.splitext(os.path.basename(__file__))[0]\n    if __package__:\n        expected_spec_name = __package__ + "." + expected_spec_name\n    assertEqual(__spec__.name, expected_spec_name)\n    assertEqual(__spec__.parent, __package__)\n    assertIdentical(__spec__.submodule_search_locations, None)\n    assertEqual(__spec__.origin, __file__)\n    if __spec__.cached is not None:\n        assertEqual(__spec__.cached, __cached__)\n# Check the sys module\nimport sys\nassertIdentical(globals(), sys.modules[__name__].__dict__)\nif __spec__ is not None:\n    # XXX: We\'re not currently making __main__ available under its real name\n    pass # assertIdentical(globals(), sys.modules[__spec__.name].__dict__)\nfrom test import test_cmd_line_script\nexample_args_list = test_cmd_line_script.example_args\nassertEqual(sys.argv[1:], example_args_list)\nprint(\'sys.argv[0]==%a\' % sys.argv[0])\nprint(\'sys.path[0]==%a\' % sys.path[0])\n# Check the working directory\nimport os\nprint(\'cwd==%a\' % os.getcwd())\n'

def _make_test_script(script_dir, script_basename, source=test_source):
    to_return = make_script(script_dir, script_basename, source)
    importlib.invalidate_caches()
    return to_return

def _make_test_zip_pkg(zip_dir, zip_basename, pkg_name, script_basename, source=test_source, depth=1):
    to_return = make_zip_pkg(zip_dir, zip_basename, pkg_name, script_basename, source, depth)
    importlib.invalidate_caches()
    return to_return

def tearDownModule():
    support.reap_children()


# --- test body ---
script = 'foo = f"""{}\nfoo"""\n'
with os_helper.temp_dir() as script_dir:
    script_name = _make_test_script(script_dir, 'script', script)
    exitcode, stdout, stderr = assert_python_failure(script_name)

    assert stderr.splitlines()[-3:] == [b'    foo = f"""{}', b'               ^', b"SyntaxError: f-string: valid expression required before '}'"]
print("CmdLineTest::test_syntaxerror_multi_line_fstring: ok")
