# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_response"
# dimension = "behavior"
# case = "test_response__test_addclosehook"
# subject = "cpython.test_urllib_response.TestResponse.test_addclosehook"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_urllib_response.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_urllib_response.py::TestResponse::test_addclosehook
"""Auto-ported test: TestResponse::test_addclosehook (CPython 3.12 oracle)."""


import socket
import urllib.response
import unittest
from test import support


if support.is_wasi:
    raise unittest.SkipTest("Cannot create socket on WASI")


self_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
self_fp = self_sock.makefile("rb")
closehook_called = False


def closehook():
    global closehook_called
    closehook_called = True


try:
    wrapped = urllib.response.addclosehook(self_fp, closehook)
    wrapped.close()

    assert self_fp.closed
    assert closehook_called
finally:
    self_sock.close()

print("TestResponse::test_addclosehook: ok")
