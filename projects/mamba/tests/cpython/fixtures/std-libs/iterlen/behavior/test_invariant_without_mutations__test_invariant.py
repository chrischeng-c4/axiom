# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "iterlen"
# dimension = "behavior"
# case = "test_invariant_without_mutations__test_invariant"
# subject = "cpython.test_iterlen.TestInvariantWithoutMutations.test_invariant"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_iterlen.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: TestInvariantWithoutMutations::test_invariant (CPython 3.12 oracle)."""

import unittest
from itertools import repeat
from test.test_iterlen import TestInvariantWithoutMutations, n


class Harness(TestInvariantWithoutMutations, unittest.TestCase):
    def setUp(self):
        self.it = repeat(None, n)


case = Harness("test_invariant")
result = unittest.TestResult()
case.run(result)

assert result.wasSuccessful(), result
assert not result.failures, result.failures
assert not result.errors, result.errors

print("TestInvariantWithoutMutations::test_invariant: ok")
