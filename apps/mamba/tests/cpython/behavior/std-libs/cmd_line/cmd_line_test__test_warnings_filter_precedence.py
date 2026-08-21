# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line"
# dimension = "behavior"
# case = "cmd_line_test__test_warnings_filter_precedence"
# subject = "cpython.test_cmd_line.CmdLineTest.test_warnings_filter_precedence"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line.py::CmdLineTest::test_warnings_filter_precedence
"""Auto-ported test: CmdLineTest::test_warnings_filter_precedence (CPython 3.12 oracle)."""


import os
import subprocess
import sys
import tempfile
import textwrap
import unittest
from test import support
from test.support import os_helper
from test.support.script_helper import spawn_python, kill_python, assert_python_ok, assert_python_failure, interpreter_requires_environment


if not support.has_subprocess_support:
    raise unittest.SkipTest('test module requires subprocess')

def _kill_python_and_exit_code(p):
    data = kill_python(p)
    returncode = p.wait()
    return (data, returncode)

def tearDownModule():
    support.reap_children()


# --- test body ---
def _test_no_stdio(streams):
    code = 'if 1:\n            import os, sys\n            for i, s in enumerate({streams}):\n                if getattr(sys, s) is not None:\n                    os._exit(i + 1)\n            os._exit(42)'.format(streams=streams)

    def preexec():
        if 'stdin' in streams:
            os.close(0)
        if 'stdout' in streams:
            os.close(1)
        if 'stderr' in streams:
            os.close(2)
    p = subprocess.Popen([sys.executable, '-E', '-c', code], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, preexec_fn=preexec)
    out, err = p.communicate()

    assert err == b''

    assert p.returncode == 42

def check_input(code, expected):
    with tempfile.NamedTemporaryFile('wb+') as stdin:
        sep = os.linesep.encode('ASCII')
        stdin.write(sep.join((b'abc', b'def')))
        stdin.flush()
        stdin.seek(0)
        with subprocess.Popen((sys.executable, '-c', code), stdin=stdin, stdout=subprocess.PIPE) as proc:
            stdout, stderr = proc.communicate()

    assert stdout.rstrip() == expected

def check_pythonmalloc(env_var, name):
    code = 'import _testcapi; print(_testcapi.pymem_getallocatorsname())'
    env = dict(os.environ)
    env.pop('PYTHONDEVMODE', None)
    if env_var is not None:
        env['PYTHONMALLOC'] = env_var
    else:
        env.pop('PYTHONMALLOC', None)
    args = (sys.executable, '-c', code)
    proc = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, universal_newlines=True, env=env)

    assert proc.stdout.rstrip() == name

    assert proc.returncode == 0

def check_warnings_filters(cmdline_option, envvar, use_pywarning=False):
    if use_pywarning:
        code = "import sys; from test.support.import_helper import import_fresh_module; warnings = import_fresh_module('warnings', blocked=['_warnings']); "
    else:
        code = 'import sys, warnings; '
    code += "print(' '.join('%s::%s' % (f[0], f[2].__name__) for f in warnings.filters))"
    args = (sys.executable, '-W', cmdline_option, '-bb', '-c', code)
    env = dict(os.environ)
    env.pop('PYTHONDEVMODE', None)
    env['PYTHONWARNINGS'] = envvar
    proc = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, universal_newlines=True, env=env)

    assert proc.returncode == 0
    return proc.stdout.rstrip()

def run_xdev(*args, check_exitcode=True, xdev=True):
    env = dict(os.environ)
    env.pop('PYTHONWARNINGS', None)
    env.pop('PYTHONDEVMODE', None)
    env.pop('PYTHONMALLOC', None)
    if xdev:
        args = (sys.executable, '-X', 'dev', *args)
    else:
        args = (sys.executable, *args)
    proc = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, universal_newlines=True, env=env)
    if check_exitcode:

        assert proc.returncode == 0
    return proc.stdout.rstrip()

def verify_valid_flag(cmd_line):
    rc, out, err = assert_python_ok(cmd_line)

    assert out == b'' or out.endswith(b'\n')

    assert b'Traceback' not in out

    assert b'Traceback' not in err
    return out
expected_filters = 'error::BytesWarning once::UserWarning always::UserWarning'
if not support.Py_DEBUG:
    expected_filters += ' default::DeprecationWarning ignore::DeprecationWarning ignore::PendingDeprecationWarning ignore::ImportWarning ignore::ResourceWarning'
out = check_warnings_filters('once::UserWarning', 'always::UserWarning')

assert out == expected_filters
out = check_warnings_filters('once::UserWarning', 'always::UserWarning', use_pywarning=True)

assert out == expected_filters
print("CmdLineTest::test_warnings_filter_precedence: ok")
