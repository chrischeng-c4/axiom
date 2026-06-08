# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_basic_formatter"
# subject = "cpython.test_string.ModuleTest.test_basic_formatter"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_basic_formatter
"""Auto-ported test: ModuleTest::test_basic_formatter (CPython 3.12 oracle)."""


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

assert fmt.format('foo') == 'foo'

assert fmt.format('foo{0}', 'bar') == 'foobar'

assert fmt.format('foo{1}{0}-{1}', 'bar', 6) == 'foo6bar-6'

try:
    fmt.format()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    string.Formatter.format()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ModuleTest::test_basic_formatter: ok")
