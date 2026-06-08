# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "iterlen"
# dimension = "behavior"
# case = "test_temporarily_immutable__test_immutable_during_iteration"
# subject = "cpython.test_iterlen.TestTemporarilyImmutable.test_immutable_during_iteration"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_iterlen.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: TestTemporarilyImmutable::test_immutable_during_iteration (CPython 3.12 oracle)."""

import unittest
from collections import deque
from test.test_iterlen import TestTemporarilyImmutable, n


class Harness(TestTemporarilyImmutable, unittest.TestCase):
    def setUp(self):
        values = deque(range(n))
        self.it = iter(values)
        self.mutate = values.pop


case = Harness("test_immutable_during_iteration")
result = unittest.TestResult()
case.run(result)

assert result.wasSuccessful(), result
assert not result.failures, result.failures
assert not result.errors, result.errors

print("TestTemporarilyImmutable::test_immutable_during_iteration: ok")
