# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audit"
# dimension = "behavior"
# case = "audit_test__test_sqlite3"
# subject = "cpython.test_audit.AuditTest.test_sqlite3"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_audit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_audit.py::AuditTest::test_sqlite3
"""Auto-ported test: AuditTest::test_sqlite3 (CPython 3.12 oracle)."""


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
sqlite3 = import_helper.import_module('sqlite3')
returncode, events, stderr = run_python('test_sqlite3')
if returncode:

    raise AssertionError(stderr)
if support.verbose:
    print(*events, sep='\n')
actual = [ev[0] for ev in events]
expected = ['sqlite3.connect', 'sqlite3.connect/handle'] * 2
if hasattr(sqlite3.Connection, 'enable_load_extension'):
    expected += ['sqlite3.enable_load_extension', 'sqlite3.load_extension']

assert actual == expected
print("AuditTest::test_sqlite3: ok")
