# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecencodings_cn"
# dimension = "behavior"
# case = "test_gb2312__test_errorhandle"
# subject = "cpython.test_codecencodings_cn.Test_GB2312.test_errorhandle"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecencodings_cn.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""GB2312 codec error handlers match CPython's multibyte codec contract."""

import io
import unittest
from test import test_codecencodings_cn


stream = io.StringIO()
suite = unittest.TestSuite([test_codecencodings_cn.Test_GB2312("test_errorhandle")])
result = unittest.TextTestRunner(stream=stream, verbosity=0).run(suite)

assert result.testsRun == 1, result.testsRun
assert not result.failures, stream.getvalue()
assert not result.errors, stream.getvalue()

if result.skipped:
    print("test_gb2312__test_errorhandle skipped:", result.skipped[0][1])
else:
    print("test_gb2312__test_errorhandle OK")
