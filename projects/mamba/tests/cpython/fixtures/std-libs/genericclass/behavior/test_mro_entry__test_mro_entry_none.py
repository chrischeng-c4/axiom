# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_mro_entry__test_mro_entry_none"
# subject = "cpython.test_genericclass.TestMROEntry.test_mro_entry_none"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericclass.py::TestMROEntry::test_mro_entry_none
"""Auto-ported test: TestMROEntry::test_mro_entry_none (CPython 3.12 oracle)."""


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
        return ()
c = C()

assert tested == []

class D(A, c, B):
    ...

assert tested[-1] == (A, c, B)

assert D.__bases__ == (A, B)

assert D.__orig_bases__ == (A, c, B)

assert D.__mro__ == (D, A, B, object)

class E(c):
    ...

assert tested[-1] == (c,)

assert E.__bases__ == (object,)

assert E.__orig_bases__ == (c,)

assert E.__mro__ == (E, object)
print("TestMROEntry::test_mro_entry_none: ok")
