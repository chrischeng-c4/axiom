# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "bigaddrspace"
# dimension = "errors"
# case = "bytes_test__test_repeat"
# subject = "cpython.test_bigaddrspace.BytesTest.test_repeat"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bigaddrspace.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_bigaddrspace.py::BytesTest::test_repeat
"""Auto-ported test: BytesTest::test_repeat (CPython 3.12 oracle)."""


import operator
import os
import sys


if os.environ.get("MAMBA_RUN_BIGADDRSPACE") != "1":
    print("BytesTest::test_repeat: skipped, set MAMBA_RUN_BIGADDRSPACE=1 to run")
    raise SystemExit(0)

try:
    value = b"x" * (sys.maxsize - 128)
    try:
        operator.mul(value, 128)
    except OverflowError:
        pass
    else:
        raise AssertionError("near-address-space bytes repeat did not raise OverflowError")
finally:
    value = None

print("BytesTest::test_repeat: ok")
