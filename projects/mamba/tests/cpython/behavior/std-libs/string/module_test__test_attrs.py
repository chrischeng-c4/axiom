# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_attrs"
# subject = "cpython.test_string.ModuleTest.test_attrs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_attrs
"""Auto-ported test: ModuleTest::test_attrs (CPython 3.12 oracle)."""


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

assert string.whitespace == ' \t\n\r\x0b\x0c'

assert string.ascii_lowercase == 'abcdefghijklmnopqrstuvwxyz'

assert string.ascii_uppercase == 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'

assert string.ascii_letters == string.ascii_lowercase + string.ascii_uppercase

assert string.digits == '0123456789'

assert string.hexdigits == string.digits + 'abcdefABCDEF'

assert string.octdigits == '01234567'

assert string.punctuation == '!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~'

assert string.printable == string.digits + string.ascii_lowercase + string.ascii_uppercase + string.punctuation + string.whitespace
print("ModuleTest::test_attrs: ok")
