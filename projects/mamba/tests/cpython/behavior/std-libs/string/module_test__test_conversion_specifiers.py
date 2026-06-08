# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_conversion_specifiers"
# subject = "cpython.test_string.ModuleTest.test_conversion_specifiers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_conversion_specifiers
"""Auto-ported test: ModuleTest::test_conversion_specifiers (CPython 3.12 oracle)."""


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

assert fmt.format('-{arg!r}-', arg='test') == "-'test'-"

assert fmt.format('{0!s}', 'test') == 'test'

try:
    fmt.format('{0!h}', 'test')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

assert fmt.format('{0!a}', 42) == '42'

assert fmt.format('{0!a}', string.ascii_letters) == "'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ'"

assert fmt.format('{0!a}', chr(255)) == "'\\xff'"

assert fmt.format('{0!a}', chr(256)) == "'\\u0100'"
print("ModuleTest::test_conversion_specifiers: ok")
