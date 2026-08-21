# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_response"
# dimension = "behavior"
# case = "test_response__test_addinfourl"
# subject = "cpython.test_urllib_response.TestResponse.test_addinfourl"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib_response.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_urllib_response.py::TestResponse::test_addinfourl
"""Auto-ported test: TestResponse::test_addinfourl (CPython 3.12 oracle)."""


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
url = 'http://www.python.org'
code = 200
infourl = urllib.response.addinfourl(self_fp, self_test_headers, url, code)

assert infourl.info() == self_test_headers

assert infourl.geturl() == url

assert infourl.getcode() == code

assert infourl.headers == self_test_headers

assert infourl.url == url

assert infourl.status == code
infourl.close()
print("TestResponse::test_addinfourl: ok")
