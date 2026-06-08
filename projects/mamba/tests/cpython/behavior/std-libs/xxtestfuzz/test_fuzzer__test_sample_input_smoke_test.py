# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xxtestfuzz"
# dimension = "behavior"
# case = "test_fuzzer__test_sample_input_smoke_test"
# subject = "cpython.test_xxtestfuzz.TestFuzzer.test_sample_input_smoke_test"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_xxtestfuzz.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_xxtestfuzz.py::TestFuzzer::test_sample_input_smoke_test
"""Auto-ported test: TestFuzzer::test_sample_input_smoke_test (CPython 3.12 oracle)."""


import faulthandler
from test.support import import_helper
import unittest


_xxtestfuzz = import_helper.import_module('_xxtestfuzz')


# --- test body ---
"""This is only a regression test: Check that it doesn't crash."""
_xxtestfuzz.run(b'')
_xxtestfuzz.run(b'\x00')
_xxtestfuzz.run(b'{')
_xxtestfuzz.run(b' ')
_xxtestfuzz.run(b'x')
_xxtestfuzz.run(b'1')
_xxtestfuzz.run(b'AAAAAAA')
_xxtestfuzz.run(b'AAAAAA\x00')
print("TestFuzzer::test_sample_input_smoke_test: ok")
