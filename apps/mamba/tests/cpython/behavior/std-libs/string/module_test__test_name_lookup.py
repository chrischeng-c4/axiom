# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_name_lookup"
# subject = "cpython.test_string.ModuleTest.test_name_lookup"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_name_lookup
"""Auto-ported test: ModuleTest::test_name_lookup (CPython 3.12 oracle)."""


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

class AnyAttr:

    def __getattr__(self, attr):
        return attr
x = AnyAttr()

assert fmt.format('{0.lumber}{0.jack}', x) == 'lumberjack'
try:
    fmt.format('{0.lumber}{0.jack}', '')
    raise AssertionError('expected AttributeError')
except AttributeError:
    pass
print("ModuleTest::test_name_lookup: ok")
