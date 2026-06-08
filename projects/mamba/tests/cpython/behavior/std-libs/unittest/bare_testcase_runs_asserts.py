# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "bare_testcase_runs_asserts"
# subject = "unittest.TestCase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase: a bare TestCase() can run individual asserts directly: assertEqual(3, 3) passes and a mismatch raises failureException inside assertRaises"""
import unittest

bare = unittest.TestCase()
bare.assertEqual(3, 3)
with bare.assertRaises(bare.failureException):
    bare.assertEqual(3, 2)
print("bare_testcase_runs_asserts OK")
