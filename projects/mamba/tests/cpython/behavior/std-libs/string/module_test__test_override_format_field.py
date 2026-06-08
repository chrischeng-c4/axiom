# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_override_format_field"
# subject = "cpython.test_string.ModuleTest.test_override_format_field"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_override_format_field
"""Auto-ported test: ModuleTest::test_override_format_field (CPython 3.12 oracle)."""


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
class CallFormatter(string.Formatter):

    def format_field(self, value, format_spec):
        return format(value(), format_spec)
fmt = CallFormatter()

assert fmt.format('*{0}*', lambda: 'result') == '*result*'
print("ModuleTest::test_override_format_field: ok")
