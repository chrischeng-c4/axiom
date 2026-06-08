# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecencodings_iso2022"
# dimension = "behavior"
# case = "test_iso2022_kr__test_chunkcoding"
# subject = "cpython.test_codecencodings_iso2022.Test_ISO2022_KR.test_chunkcoding"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_codecencodings_iso2022.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: Test_ISO2022_KR::test_chunkcoding (CPython 3.12 oracle)."""

import unittest
from test.test_codecencodings_iso2022 import Test_ISO2022_KR


case = Test_ISO2022_KR("test_chunkcoding")
result = unittest.TestResult()
case.run(result)

assert result.wasSuccessful(), result
assert len(result.skipped) == 1, result.skipped
assert result.skipped[0][0] is case
assert result.skipped[0][1] == 'iso2022_kr.txt cannot be used to test "chunk coding"'

print("Test_ISO2022_KR::test_chunkcoding skipped: ok")
