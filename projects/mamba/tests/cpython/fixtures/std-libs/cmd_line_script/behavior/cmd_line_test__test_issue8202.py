# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line_script"
# dimension = "behavior"
# case = "cmd_line_test__test_issue8202"
# subject = "cpython.test_cmd_line_script.CmdLineTest.test_issue8202"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line_script.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line_script.py::CmdLineTest::test_issue8202
"""Auto-ported test: CmdLineTest::test_issue8202 (CPython 3.12 oracle)."""


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
def _check_import_error(script_exec_args, expected_msg, *cmd_line_switches, cwd=None, **env_vars):
    if isinstance(script_exec_args, str):
        script_exec_args = (script_exec_args,)
    else:
        script_exec_args = tuple(script_exec_args)
    run_args = cmd_line_switches + script_exec_args
    rc, out, err = assert_python_failure(*run_args, __isolated=False, __cwd=cwd, **env_vars)
    if verbose > 1:
        print(f'Output from test script {script_exec_args!r:}')
        print(repr(err))
        print('Expected output: %r' % expected_msg)

    assert expected_msg.encode('utf-8') in err

def _check_output(script_name, exit_code, data, expected_file, expected_argv0, expected_path0, expected_package, expected_loader, expected_cwd=None):
    if verbose > 1:
        print('Output from test script %r:' % script_name)
        print(repr(data))

    assert exit_code == 0
    printed_loader = '__loader__==%a' % expected_loader
    printed_file = '__file__==%a' % expected_file
    printed_package = '__package__==%r' % expected_package
    printed_argv0 = 'sys.argv[0]==%a' % expected_argv0
    printed_path0 = 'sys.path[0]==%a' % expected_path0
    if expected_cwd is None:
        expected_cwd = os.getcwd()
    printed_cwd = 'cwd==%a' % expected_cwd
    if verbose > 1:
        print('Expected output:')
        print(printed_file)
        print(printed_package)
        print(printed_argv0)
        print(printed_cwd)

    assert printed_loader.encode('utf-8') in data

    assert printed_file.encode('utf-8') in data

    assert printed_package.encode('utf-8') in data

    assert printed_argv0.encode('utf-8') in data
    if not sys.flags.safe_path:

        assert printed_path0.encode('utf-8') in data

    assert printed_cwd.encode('utf-8') in data

def _check_script(script_exec_args, expected_file, expected_argv0, expected_path0, expected_package, expected_loader, *cmd_line_switches, cwd=None, **env_vars):
    if isinstance(script_exec_args, str):
        script_exec_args = [script_exec_args]
    run_args = [*support.optim_args_from_interpreter_flags(), *cmd_line_switches, *script_exec_args, *example_args]
    rc, out, err = assert_python_ok(*run_args, __isolated=False, __cwd=cwd, **env_vars)
    _check_output(script_exec_args, rc, out + err, expected_file, expected_argv0, expected_path0, expected_package, expected_loader, cwd)

def check_dash_m_failure(*args):
    rc, out, err = assert_python_failure('-m', *args, __isolated=False)
    if verbose > 1:
        print(repr(out))

    assert rc == 1
    return err

def check_repl_stderr_flush(separate_stderr=False):
    with interactive_python(separate_stderr) as p:
        p.stdin.write(b'1/0\n')
        p.stdin.flush()
        stderr = p.stderr if separate_stderr else p.stdout

        assert b'Traceback ' in stderr.readline()

        assert b'File "<stdin>"' in stderr.readline()

        assert b'ZeroDivisionError' in stderr.readline()

def check_repl_stdout_flush(separate_stderr=False):
    with interactive_python(separate_stderr) as p:
        p.stdin.write(b"print('foo')\n")
        p.stdin.flush()

        assert b'foo' == p.stdout.readline().strip()

def interactive_python(separate_stderr=False):
    if separate_stderr:
        p = spawn_python('-i', stderr=subprocess.PIPE)
        stderr = p.stderr
    else:
        p = spawn_python('-i', stderr=subprocess.STDOUT)
        stderr = p.stdout
    try:
        while True:
            data = stderr.read(4)
            if data == b'>>> ':
                break
            stderr.readline()
        yield p
    finally:
        kill_python(p)
        stderr.close()

def setup_test_pkg(*args):
    with os_helper.temp_dir() as script_dir, os_helper.change_cwd(path=script_dir):
        pkg_dir = os.path.join(script_dir, 'test_pkg')
        make_pkg(pkg_dir, *args)
        yield pkg_dir
with os_helper.temp_dir() as script_dir:
    with os_helper.change_cwd(path=script_dir):
        pkg_dir = os.path.join(script_dir, 'test_pkg')
        make_pkg(pkg_dir, "import sys; print('init_argv0==%r' % sys.argv[0])")
        script_name = _make_test_script(pkg_dir, 'script')
        rc, out, err = assert_python_ok('-m', 'test_pkg.script', *example_args, __isolated=False)
        if verbose > 1:
            print(repr(out))
        expected = 'init_argv0==%r' % '-m'

        assert expected.encode('utf-8') in out
        _check_output(script_name, rc, out, script_name, script_name, script_dir, 'test_pkg', importlib.machinery.SourceFileLoader)
print("CmdLineTest::test_issue8202: ok")
