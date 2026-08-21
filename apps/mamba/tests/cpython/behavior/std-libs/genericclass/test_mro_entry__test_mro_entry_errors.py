# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_mro_entry__test_mro_entry_errors"
# subject = "cpython.test_genericclass.TestMROEntry.test_mro_entry_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericclass.py::TestMROEntry::test_mro_entry_errors
"""Auto-ported test: TestMROEntry::test_mro_entry_errors (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
class C_too_many:

    def __mro_entries__(self, bases, something, other):
        return ()
c = C_too_many()
try:

    class D(c):
        ...
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class C_too_few:

    def __mro_entries__(self):
        return ()
d = C_too_few()
try:

    class E(d):
        ...
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestMROEntry::test_mro_entry_errors: ok")
