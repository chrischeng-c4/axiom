# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "surface"
# case = "skiptest_subclasses_exception"
# subject = "unittest.SkipTest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.SkipTest: unittest.SkipTest is a subclass of the builtin Exception"""
import unittest

assert issubclass(unittest.SkipTest, Exception)
print("skiptest_subclasses_exception OK")
