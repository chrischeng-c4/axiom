# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "failureexception_is_assertionerror"
# subject = "unittest.TestCase.failureException"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase.failureException: TestCase.failureException defaults to the builtin AssertionError"""
import unittest

assert unittest.TestCase.failureException is AssertionError
print("failureexception_is_assertionerror OK")
