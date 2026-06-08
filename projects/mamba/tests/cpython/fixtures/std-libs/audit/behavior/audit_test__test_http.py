# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audit"
# dimension = "behavior"
# case = "audit_test__test_http"
# subject = "cpython.test_audit.AuditTest.test_http"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_audit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_audit.py::AuditTest::test_http
"""Auto-ported test: AuditTest::test_http (CPython 3.12 oracle)."""


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
import_helper.import_module('http.client')
returncode, events, stderr = run_python('test_http_client')
if returncode:

    raise AssertionError(stderr)
if support.verbose:
    print(*events, sep='\n')

assert events[0][0] == 'http.client.connect'

assert events[0][2] == 'www.python.org 80'

assert events[1][0] == 'http.client.send'
if events[1][2] != '[cannot send]':

    assert 'HTTP' in events[1][2]
print("AuditTest::test_http: ok")
