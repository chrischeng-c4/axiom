# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "behavior"
# case = "call_delegates_to_run"
# subject = "unittest.TestCase.__call__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase.__call__: calling the instance delegates to run() with the supplied result and returns run()'s return value"""
import unittest

result_in = unittest.TestResult()
result_out = unittest.TestResult()


class Delegating(unittest.TestCase):
    def test(self):
        pass

    def run(self, result=None):
        # The result passed by __call__ is exactly the one we supplied.
        assert result is result_in
        return result_out


retval = Delegating("test")(result_in)
assert retval is result_out
print("call_delegates_to_run OK")
