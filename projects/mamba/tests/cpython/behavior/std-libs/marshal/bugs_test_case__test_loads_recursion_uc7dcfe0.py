# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "behavior"
# case = "bugs_test_case__test_loads_recursion_uc7dcfe0"
# subject = "cpython.test_marshal.BugsTestCase.test_loads_recursion"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_marshal.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import array
import io
import marshal
import sys
import os
import types
import textwrap

def run_tests(N, check):
    check(b')\x01' * N + b'N')
    check(b'(\x01\x00\x00\x00' * N + b'N')
    check(b'[\x01\x00\x00\x00' * N + b'N')
    check(b'{N' * N + b'N' + b'0' * N)
    check(b'>\x01\x00\x00\x00' * N + b'N')
run_tests(100, marshal.loads)

def check(s):
    try:
        marshal.loads(s)
        raise AssertionError('assertRaises: no raise')
    except ValueError:
        pass
run_tests(2 ** 20, check)

print("BugsTestCase::test_loads_recursion: ok")
