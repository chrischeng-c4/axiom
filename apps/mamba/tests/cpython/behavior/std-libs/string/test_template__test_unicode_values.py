# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_unicode_values"
# subject = "cpython.test_string.TestTemplate.test_unicode_values"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_unicode_values
"""Auto-ported test: TestTemplate::test_unicode_values (CPython 3.12 oracle)."""


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
s = Template('$who likes $what')
d = dict(who='tÿm', what='fþ\x0ced')

assert s.substitute(d) == 'tÿm likes fþ\x0ced'
print("TestTemplate::test_unicode_values: ok")
