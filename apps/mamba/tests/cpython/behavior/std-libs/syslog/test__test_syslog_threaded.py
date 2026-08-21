# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "syslog"
# dimension = "behavior"
# case = "test__test_syslog_threaded"
# subject = "cpython.test_syslog.Test.test_syslog_threaded"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_syslog.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_syslog.py::Test::test_syslog_threaded
"""Auto-ported test: Test::test_syslog_threaded (CPython 3.12 oracle)."""


from test.support import import_helper, threading_helper
from test import support
import sys
import threading
import time
import unittest
from textwrap import dedent


syslog = import_helper.import_module('syslog')


# --- test body ---
start = threading.Event()
stop = False

def opener():
    start.wait(10)
    i = 1
    while not stop:
        syslog.openlog(f'python-test-{i}')
        i += 1

def logger():
    start.wait(10)
    while not stop:
        syslog.syslog('test message from python test_syslog')
orig_si = sys.getswitchinterval()
support.setswitchinterval(1e-09)
try:
    threads = [threading.Thread(target=opener)]
    threads += [threading.Thread(target=logger) for k in range(10)]
    with threading_helper.start_threads(threads):
        start.set()
        time.sleep(0.1)
        stop = True
finally:
    sys.setswitchinterval(orig_si)
print("Test::test_syslog_threaded: ok")
