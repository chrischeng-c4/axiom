# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contains"
# dimension = "behavior"
# case = "test_contains__test_block_fallback"
# subject = "cpython.test_contains.TestContains.test_block_fallback"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contains.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_contains.py::TestContains::test_block_fallback
"""Auto-ported test: TestContains::test_block_fallback (CPython 3.12 oracle)."""


from collections import deque
import unittest
from test.support import NEVER_EQ


class base_set:

    def __init__(self, el):
        self.el = el

class myset(base_set):

    def __contains__(self, el):
        return self.el == el

class seq(base_set):

    def __getitem__(self, n):
        return [self.el][n]


# --- test body ---
class ByContains(object):

    def __contains__(self, other):
        return False
c = ByContains()

class BlockContains(ByContains):
    """Is not a container

            This class is a perfectly good iterable (as tested by
            list(bc)), as well as inheriting from a perfectly good
            container, but __contains__ = None prevents the usual
            fallback to iteration in the container protocol. That
            is, normally, 0 in bc would fall back to the equivalent
            of any(x==0 for x in bc), but here it's blocked from
            doing so.
            """

    def __iter__(self):
        while False:
            yield None
    __contains__ = None
bc = BlockContains()

assert not 0 in c

assert not 0 in list(bc)

try:
    (lambda: 0 in bc)()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestContains::test_block_fallback: ok")
