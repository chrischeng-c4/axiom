# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "profile"
# dimension = "behavior"
# case = "profile_test__test_cprofile"
# subject = "cpython.test_profile.ProfileTest.test_cprofile"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_profile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_profile.py::ProfileTest::test_cprofile
"""Auto-ported test: ProfileTest::test_cprofile (CPython 3.12 oracle)."""

import unittest
from test.test_profile import ProfileTest


case = ProfileTest("test_cprofile")
result = unittest.TestResult()
case.run(result)
assert result.wasSuccessful(), result
assert not result.failures, result.failures
assert not result.errors, result.errors

print("ProfileTest::test_cprofile deterministic stats boundary: ok")
