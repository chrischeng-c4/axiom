# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_override_convert_field"
# subject = "cpython.test_string.ModuleTest.test_override_convert_field"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_override_convert_field
"""Auto-ported test: ModuleTest::test_override_convert_field (CPython 3.12 oracle)."""


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
class XFormatter(string.Formatter):

    def convert_field(self, value, conversion):
        if conversion == 'x':
            return None
        return super().convert_field(value, conversion)
fmt = XFormatter()

assert fmt.format('{0!r}:{0!x}', 'foo', 'foo') == "'foo':None"
print("ModuleTest::test_override_convert_field: ok")
