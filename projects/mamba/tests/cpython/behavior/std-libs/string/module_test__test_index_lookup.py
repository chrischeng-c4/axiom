# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_index_lookup"
# subject = "cpython.test_string.ModuleTest.test_index_lookup"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_index_lookup
"""Auto-ported test: ModuleTest::test_index_lookup (CPython 3.12 oracle)."""


import unittest
import string
from string import Template


class Bag:
    pass

class Mapping:

    def __getitem__(self, name):
        obj = self
        for part in name.split('.'):
            try:
                obj = getattr(obj, part)
            except AttributeError:
                raise KeyError(name)
        return obj


# --- test body ---
fmt = string.Formatter()
lookup = ['eggs', 'and', 'spam']

assert fmt.format('{0[2]}{0[0]}', lookup) == 'spameggs'
try:
    fmt.format('{0[2]}{0[0]}', [])
    raise AssertionError('expected IndexError')
except IndexError:
    pass
try:
    fmt.format('{0[2]}{0[0]}', {})
    raise AssertionError('expected KeyError')
except KeyError:
    pass
print("ModuleTest::test_index_lookup: ok")
