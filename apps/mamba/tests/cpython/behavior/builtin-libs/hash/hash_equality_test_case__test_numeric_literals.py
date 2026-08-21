# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "hash"
# dimension = "behavior"
# case = "hash_equality_test_case__test_numeric_literals"
# subject = "cpython.test.test_hash.HashEqualityTestCase.test_numeric_literals"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hash.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_hash.py::HashEqualityTestCase::test_numeric_literals
"""Auto-ported test: HashEqualityTestCase::test_numeric_literals (CPython 3.12 oracle)."""


def same_hash(*objects):
    hashes = [hash(obj) for obj in objects]
    for value in hashes[1:]:
        assert value == hashes[0], (objects, hashes)


same_hash(1, 1, 1.0, 1.0 + 0.0j)
same_hash(0, 0.0, 0.0 + 0.0j)
same_hash(-1, -1.0, -1.0 + 0.0j)
same_hash(-2, -2.0, -2.0 + 0.0j)

print("HashEqualityTestCase::test_numeric_literals: ok")
