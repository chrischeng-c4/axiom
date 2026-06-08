# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audit"
# dimension = "behavior"
# case = "audit_test__test_open"
# subject = "cpython.test_audit.AuditTest.test_open"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_audit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_audit.py::AuditTest::test_open
"""Auto-ported test: AuditTest::test_open (CPython 3.12 oracle)."""


import subprocess
import sys
import unittest
from test import support
from test.support import import_helper
from test.support import os_helper


'Tests for sys.audit and sys.addaudithook\n'

if not hasattr(sys, 'addaudithook') or not hasattr(sys, 'audit'):
    raise unittest.SkipTest('test only relevant when sys.audit is available')

AUDIT_TESTS_PY = support.findfile('audit-tests.py')


# --- test body ---
maxDiff = None

def do_test(*args):
    with subprocess.Popen([sys.executable, '-X utf8', AUDIT_TESTS_PY, *args], encoding='utf-8', errors='backslashreplace', stdout=subprocess.PIPE, stderr=subprocess.PIPE) as p:
        p.wait()
        sys.stdout.writelines(p.stdout)
        sys.stderr.writelines(p.stderr)
        if p.returncode:

            raise AssertionError(''.join(p.stderr))

def run_python(*args):
    events = []
    with subprocess.Popen([sys.executable, '-X utf8', AUDIT_TESTS_PY, *args], encoding='utf-8', stdout=subprocess.PIPE, stderr=subprocess.PIPE) as p:
        p.wait()
        sys.stderr.writelines(p.stderr)
        return (p.returncode, [line.strip().partition(' ') for line in p.stdout], ''.join(p.stderr))
do_test('test_open', os_helper.TESTFN)
print("AuditTest::test_open: ok")
