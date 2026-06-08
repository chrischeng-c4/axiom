# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "id_ends_with_bound_method_name"
# subject = "unittest.TestCase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase: TestCase('test_a').id() ends with '.<ClassName>.test_a'; the named ctor selects that test method"""
import unittest


class Sample(unittest.TestCase):
    def test_a(self):
        pass

    def runTest(self):
        pass


assert Sample("test_a").id().endswith(".Sample.test_a")
print("id_ends_with_bound_method_name OK")
