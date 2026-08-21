# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_mro_entry__test_mro_entry_errors_2"
# subject = "cpython.test_genericclass.TestMROEntry.test_mro_entry_errors_2"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericclass.py::TestMROEntry::test_mro_entry_errors_2
"""Auto-ported test: TestMROEntry::test_mro_entry_errors_2 (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
class C_not_callable:
    __mro_entries__ = 'Surprise!'
c = C_not_callable()
try:

    class D(c):
        ...
    raise AssertionError('expected TypeError')
except TypeError:
    pass

class C_not_tuple:

    def __mro_entries__(self):
        return object
c = C_not_tuple()
try:

    class E(c):
        ...
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestMROEntry::test_mro_entry_errors_2: ok")
