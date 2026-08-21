# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_override_parse"
# subject = "cpython.test_string.ModuleTest.test_override_parse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_override_parse
"""Auto-ported test: ModuleTest::test_override_parse (CPython 3.12 oracle)."""


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
class BarFormatter(string.Formatter):

    def parse(self, format_string):
        for field in format_string.split('|'):
            if field[0] == '+':
                field_name, _, format_spec = field[1:].partition(':')
                yield ('', field_name, format_spec, None)
            else:
                yield (field, None, None, None)
fmt = BarFormatter()

assert fmt.format('*|+0:^10s|*', 'foo') == '*   foo    *'
print("ModuleTest::test_override_parse: ok")
