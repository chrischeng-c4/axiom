# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "module_test__test_override_get_value"
# subject = "cpython.test_string.ModuleTest.test_override_get_value"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_string.py::ModuleTest::test_override_get_value
"""Auto-ported test: ModuleTest::test_override_get_value (CPython 3.12 oracle)."""


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
class NamespaceFormatter(string.Formatter):

    def __init__(self, namespace={}):
        string.Formatter.__init__(self)
        self.namespace = namespace

    def get_value(self, key, args, kwds):
        if isinstance(key, str):
            try:
                return kwds[key]
            except KeyError:
                return self.namespace[key]
        else:
            string.Formatter.get_value(key, args, kwds)
fmt = NamespaceFormatter({'greeting': 'hello'})

assert fmt.format('{greeting}, world!') == 'hello, world!'
print("ModuleTest::test_override_get_value: ok")
