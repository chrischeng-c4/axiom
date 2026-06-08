# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_response"
# dimension = "behavior"
# case = "test_response__test_with"
# subject = "cpython.test_urllib_response.TestResponse.test_with"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib_response.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_urllib_response.py::TestResponse::test_with
"""Auto-ported test: TestResponse::test_with (CPython 3.12 oracle)."""


import socket
import tempfile
import urllib.response
import unittest
from test import support


'Unit tests for code in urllib.response.'

if support.is_wasi:
    raise unittest.SkipTest('Cannot create socket on WASI')


# --- test body ---
self_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
self_fp = self_sock.makefile('rb')
self_test_headers = {'Host': 'www.python.org', 'Connection': 'close'}
addbase = urllib.response.addbase(self_fp)

assert isinstance(addbase, tempfile._TemporaryFileWrapper)

def f():
    with addbase as spam:
        pass

assert not self_fp.closed
f()

assert self_fp.closed

try:
    f()
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("TestResponse::test_with: ok")
