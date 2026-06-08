# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_mro_entry__test_mro_entry"
# subject = "cpython.test_genericclass.TestMROEntry.test_mro_entry"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericclass.py::TestMROEntry::test_mro_entry
"""Auto-ported test: TestMROEntry::test_mro_entry (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
tested = []

class A:
    ...

class B:
    ...

class C:

    def __mro_entries__(self, bases):
        tested.append(bases)
        return (self.__class__,)
c = C()

assert tested == []

class D(A, c, B):
    ...

assert tested[-1] == (A, c, B)

assert D.__bases__ == (A, C, B)

assert D.__orig_bases__ == (A, c, B)

assert D.__mro__ == (D, A, C, B, object)
d = D()

class E(d):
    ...

assert tested[-1] == (d,)

assert E.__bases__ == (D,)
print("TestMROEntry::test_mro_entry: ok")
