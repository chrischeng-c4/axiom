# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecmaps_tw"
# dimension = "behavior"
# case = "test_big5_map__test_errorhandle"
# subject = "cpython.test_codecmaps_tw.TestBIG5Map.test_errorhandle"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecmaps_tw.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Big5 mapping-table error handlers match CPython's oracle."""

import io
import unittest
from test import test_codecmaps_tw


stream = io.StringIO()
suite = unittest.TestSuite([test_codecmaps_tw.TestBIG5Map("test_errorhandle")])
result = unittest.TextTestRunner(stream=stream, verbosity=0).run(suite)

assert result.testsRun == 1, result.testsRun
assert not result.failures, stream.getvalue()
assert not result.errors, stream.getvalue()

if result.skipped:
    print("test_big5_map__test_errorhandle skipped:", result.skipped[0][1])
else:
    print("test_big5_map__test_errorhandle OK")
