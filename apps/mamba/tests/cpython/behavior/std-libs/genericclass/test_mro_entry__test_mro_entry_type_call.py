# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericclass"
# dimension = "behavior"
# case = "test_mro_entry__test_mro_entry_type_call"
# subject = "cpython.test_genericclass.TestMROEntry.test_mro_entry_type_call"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_genericclass.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_genericclass.py::TestMROEntry::test_mro_entry_type_call
"""Auto-ported test: TestMROEntry::test_mro_entry_type_call (CPython 3.12 oracle)."""


import unittest
from test import support


# --- test body ---
class C:

    def __mro_entries__(self, bases):
        return ()
c = C()
try:
    type('Bad', (c,), {})
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('MRO entry resolution; use types.new_class()', str(_aR_e))
print("TestMROEntry::test_mro_entry_type_call: ok")
