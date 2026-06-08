# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "osx_env"
# dimension = "behavior"
# case = "osx_environment_variable_test_case__test_pythonexecutable_sets_sys_executable"
# subject = "cpython.test_osx_env.OSXEnvironmentVariableTestCase.test_pythonexecutable_sets_sys_executable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_osx_env.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_osx_env.py::OSXEnvironmentVariableTestCase::test_pythonexecutable_sets_sys_executable
"""Auto-ported test: OSXEnvironmentVariableTestCase::test_pythonexecutable_sets_sys_executable (CPython 3.12 oracle)."""


from test.support.os_helper import EnvironmentVarGuard
import subprocess
import sys
import sysconfig
import unittest


'\nTest suite for OS X interpreter environment variables.\n'


# --- test body ---
def _check_sys(ev, cond, sv, val=sys.executable + 'dummy'):
    with EnvironmentVarGuard() as evg:
        subpc = [str(sys.executable), '-c', 'import sys; sys.exit(2 if "%s" %s %s else 3)' % (val, cond, sv)]
        evg.unset(ev)
        rc = subprocess.call(subpc)

        assert rc == 3
        evg.set(ev, val)
        rc = subprocess.call(subpc)

        assert rc == 2
_check_sys('PYTHONEXECUTABLE', '==', 'sys.executable')
print("OSXEnvironmentVariableTestCase::test_pythonexecutable_sets_sys_executable: ok")
