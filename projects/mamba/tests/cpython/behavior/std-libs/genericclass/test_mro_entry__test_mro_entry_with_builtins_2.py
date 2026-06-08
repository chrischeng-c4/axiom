# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_mro_entry__test_mro_entry_with_builtins_2"
# subject = "cpython.test_genericclass.TestMROEntry.test_mro_entry_with_builtins_2"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericclass.py::TestMROEntry::test_mro_entry_with_builtins_2
"""Auto-ported test: TestMROEntry::test_mro_entry_with_builtins_2 (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
tested = []

class C:

    def __mro_entries__(self, bases):
        tested.append(bases)
        return (C,)
c = C()

assert tested == []

class D(c, dict):
    ...

assert tested[-1] == (c, dict)

assert D.__bases__ == (C, dict)

assert D.__orig_bases__ == (c, dict)

assert D.__mro__ == (D, C, dict, object)
print("TestMROEntry::test_mro_entry_with_builtins_2: ok")
