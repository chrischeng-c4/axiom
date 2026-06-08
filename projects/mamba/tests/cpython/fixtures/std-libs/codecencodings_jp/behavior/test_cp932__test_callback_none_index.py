# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecencodings_jp"
# dimension = "behavior"
# case = "test_cp932__test_callback_none_index"
# subject = "cpython.test_codecencodings_jp.Test_CP932.test_callback_None_index"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecencodings_jp.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""CP932 codec callback None-index handling matches CPython."""

import io
import unittest
from test import test_codecencodings_jp


stream = io.StringIO()
suite = unittest.TestSuite([test_codecencodings_jp.Test_CP932("test_callback_None_index")])
result = unittest.TextTestRunner(stream=stream, verbosity=0).run(suite)

assert result.testsRun == 1, result.testsRun
assert not result.failures, stream.getvalue()
assert not result.errors, stream.getvalue()

if result.skipped:
    print("test_cp932__test_callback_none_index skipped:", result.skipped[0][1])
else:
    print("test_cp932__test_callback_none_index OK")
