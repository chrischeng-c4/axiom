# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecmaps_jp"
# dimension = "behavior"
# case = "test_cp932_map__test_errorhandle"
# subject = "cpython.test_codecmaps_jp.TestCP932Map.test_errorhandle"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecmaps_jp.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""CP932 mapping-table error handlers match CPython's oracle."""

import io
import unittest
from test import test_codecmaps_jp


stream = io.StringIO()
suite = unittest.TestSuite([test_codecmaps_jp.TestCP932Map("test_errorhandle")])
result = unittest.TextTestRunner(stream=stream, verbosity=0).run(suite)

assert result.testsRun == 1, result.testsRun
assert not result.failures, stream.getvalue()
assert not result.errors, stream.getvalue()

if result.skipped:
    print("test_cp932_map__test_errorhandle skipped:", result.skipped[0][1])
else:
    print("test_cp932_map__test_errorhandle OK")
