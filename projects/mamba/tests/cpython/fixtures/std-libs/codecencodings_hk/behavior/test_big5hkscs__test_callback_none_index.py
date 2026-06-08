# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecencodings_hk"
# dimension = "behavior"
# case = "test_big5hkscs__test_callback_none_index"
# subject = "cpython.test_codecencodings_hk.Test_Big5HKSCS.test_callback_None_index"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecencodings_hk.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Big5-HKSCS codec callback None-index handling matches CPython."""

import io
import unittest
from test import test_codecencodings_hk


stream = io.StringIO()
suite = unittest.TestSuite(
    [test_codecencodings_hk.Test_Big5HKSCS("test_callback_None_index")]
)
result = unittest.TextTestRunner(stream=stream, verbosity=0).run(suite)

assert result.testsRun == 1, result.testsRun
assert not result.failures, stream.getvalue()
assert not result.errors, stream.getvalue()

if result.skipped:
    print("test_big5hkscs__test_callback_none_index skipped:", result.skipped[0][1])
else:
    print("test_big5hkscs__test_callback_none_index OK")
