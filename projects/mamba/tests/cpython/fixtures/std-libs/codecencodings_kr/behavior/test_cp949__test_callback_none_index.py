# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecencodings_kr"
# dimension = "behavior"
# case = "test_cp949__test_callback_none_index"
# subject = "cpython.test_codecencodings_kr.Test_CP949.test_callback_None_index"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecencodings_kr.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""CP949 codec callback None-index handling matches CPython."""

import io
import unittest
from test import test_codecencodings_kr


stream = io.StringIO()
suite = unittest.TestSuite([test_codecencodings_kr.Test_CP949("test_callback_None_index")])
result = unittest.TextTestRunner(stream=stream, verbosity=0).run(suite)

assert result.testsRun == 1, result.testsRun
assert not result.failures, stream.getvalue()
assert not result.errors, stream.getvalue()

if result.skipped:
    print("test_cp949__test_callback_none_index skipped:", result.skipped[0][1])
else:
    print("test_cp949__test_callback_none_index OK")
