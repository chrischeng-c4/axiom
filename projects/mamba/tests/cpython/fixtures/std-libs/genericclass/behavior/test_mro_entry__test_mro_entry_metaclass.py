# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_mro_entry__test_mro_entry_metaclass"
# subject = "cpython.test_genericclass.TestMROEntry.test_mro_entry_metaclass"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericclass.py::TestMROEntry::test_mro_entry_metaclass
"""Auto-ported test: TestMROEntry::test_mro_entry_metaclass (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
meta_args = []

class Meta(type):

    def __new__(mcls, name, bases, ns):
        meta_args.extend([mcls, name, bases, ns])
        return super().__new__(mcls, name, bases, ns)

class A:
    ...

class C:

    def __mro_entries__(self, bases):
        return (A,)
c = C()

class D(c, metaclass=Meta):
    x = 1

assert meta_args[0] == Meta

assert meta_args[1] == 'D'

assert meta_args[2] == (A,)

assert meta_args[3]['x'] == 1

assert D.__bases__ == (A,)

assert D.__orig_bases__ == (c,)

assert D.__mro__ == (D, A, object)

assert D.__class__ == Meta
print("TestMROEntry::test_mro_entry_metaclass: ok")
