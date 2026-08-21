# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "range"
# dimension = "behavior"
# case = "range_test__test_iterator_unpickle_compat"
# subject = "cpython.test_range.RangeTest.test_iterator_unpickle_compat"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_range.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_range.py::RangeTest::test_iterator_unpickle_compat
"""Auto-ported test: RangeTest::test_iterator_unpickle_compat (CPython 3.12 oracle)."""


import unittest
import sys
import pickle
import itertools
from test.support import ALWAYS_EQ


def pyrange(start, stop, step):
    if (start - stop) // step < 0:
        stop += (start - stop) % step
        while start != stop:
            yield start
            start += step

def pyrange_reversed(start, stop, step):
    stop += (start - stop) % step
    return pyrange(stop - step, start - step, -step)


# --- test body ---
testcases = [b'c__builtin__\niter\n(c__builtin__\nxrange\n(I10\nI20\nI2\ntRtRI2\nb.', b'c__builtin__\niter\n(c__builtin__\nxrange\n(K\nK\x14K\x02tRtRK\x02b.', b'\x80\x02c__builtin__\niter\nc__builtin__\nxrange\nK\nK\x14K\x02\x87R\x85RK\x02b.', b'\x80\x03cbuiltins\niter\ncbuiltins\nrange\nK\nK\x14K\x02\x87R\x85RK\x02b.', b'\x80\x04\x951\x00\x00\x00\x00\x00\x00\x00\x8c\x08builtins\x8c\x04iter\x93\x8c\x08builtins\x8c\x05range\x93K\nK\x14K\x02\x87R\x85RK\x02b.', b'c__builtin__\niter\n(c__builtin__\nxrange\n(L-36893488147419103232L\nI20\nI2\ntRtRL18446744073709551623L\nb.', b'c__builtin__\niter\n(c__builtin__\nxrange\n(L-36893488147419103232L\nK\x14K\x02tRtRL18446744073709551623L\nb.', b'\x80\x02c__builtin__\niter\nc__builtin__\nxrange\n\x8a\t\x00\x00\x00\x00\x00\x00\x00\x00\xfeK\x14K\x02\x87R\x85R\x8a\t\x07\x00\x00\x00\x00\x00\x00\x00\x01b.', b'\x80\x03cbuiltins\niter\ncbuiltins\nrange\n\x8a\t\x00\x00\x00\x00\x00\x00\x00\x00\xfeK\x14K\x02\x87R\x85R\x8a\t\x07\x00\x00\x00\x00\x00\x00\x00\x01b.', b'\x80\x04\x95C\x00\x00\x00\x00\x00\x00\x00\x8c\x08builtins\x8c\x04iter\x93\x8c\x08builtins\x8c\x05range\x93\x8a\t\x00\x00\x00\x00\x00\x00\x00\x00\xfeK\x14K\x02\x87R\x85R\x8a\t\x07\x00\x00\x00\x00\x00\x00\x00\x01b.']
for t in testcases:
    it = pickle.loads(t)

    assert list(it) == [14, 16, 18]
print("RangeTest::test_iterator_unpickle_compat: ok")
