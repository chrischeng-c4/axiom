# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "file_cookie_jar_tests__test_lwp_filepermissions_uc1e93de"
# subject = "cpython.test_http_cookiejar.FileCookieJarTests.test_lwp_filepermissions"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_http_cookiejar
_suite = unittest.defaultTestLoader.loadTestsFromName("FileCookieJarTests.test_lwp_filepermissions", test_http_cookiejar)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FileCookieJarTests.test_lwp_filepermissions did not pass"
print("FileCookieJarTests::test_lwp_filepermissions: ok")
