# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_mro_entry__test_mro_entry_with_builtins"
# subject = "cpython.test_genericclass.TestMROEntry.test_mro_entry_with_builtins"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericclass.py::TestMROEntry::test_mro_entry_with_builtins
"""Auto-ported test: TestMROEntry::test_mro_entry_with_builtins (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
tested = []

class A:
    ...

class C:

    def __mro_entries__(self, bases):
        tested.append(bases)
        return (dict,)
c = C()

assert tested == []

class D(A, c):
    ...

assert tested[-1] == (A, c)

assert D.__bases__ == (A, dict)

assert D.__orig_bases__ == (A, c)

assert D.__mro__ == (D, A, dict, object)
print("TestMROEntry::test_mro_entry_with_builtins: ok")
