# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_mro_entry__test_mro_entry_signature"
# subject = "cpython.test_genericclass.TestMROEntry.test_mro_entry_signature"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericclass.py::TestMROEntry::test_mro_entry_signature
"""Auto-ported test: TestMROEntry::test_mro_entry_signature (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
tested = []

class B:
    ...

class C:

    def __mro_entries__(self, *args, **kwargs):
        tested.extend([args, kwargs])
        return (C,)
c = C()

assert tested == []

class D(B, c):
    ...

assert tested[0] == ((B, c),)

assert tested[1] == {}
print("TestMROEntry::test_mro_entry_signature: ok")
