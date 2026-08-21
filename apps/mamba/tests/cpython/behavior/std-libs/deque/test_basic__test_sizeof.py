# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "deque"
# dimension = "behavior"
# case = "test_basic__test_sizeof"
# subject = "cpython.test_deque.TestBasic.test_sizeof"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_deque.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: TestBasic::test_sizeof (CPython 3.12 oracle)."""

import struct
import unittest
from collections import deque
from test import support


testcase = unittest.TestCase()
max_free_blocks = 16
block_len = 64
basesize = support.calcvobjsize("2P5n%dPP" % max_free_blocks)
blocksize = struct.calcsize("P%dPP" % block_len)

assert object.__sizeof__(deque()) == basesize

for value, expected in [
    (deque(), basesize + blocksize),
    (deque("a"), basesize + blocksize),
    (deque("a" * (block_len - 1)), basesize + blocksize),
    (deque("a" * block_len), basesize + 2 * blocksize),
    (deque("a" * (42 * block_len)), basesize + 43 * blocksize),
]:
    support.check_sizeof(testcase, value, expected)

print("TestBasic::test_sizeof: ok")
