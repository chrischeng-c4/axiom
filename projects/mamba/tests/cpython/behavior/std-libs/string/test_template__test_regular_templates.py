# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_regular_templates"
# subject = "cpython.test_string.TestTemplate.test_regular_templates"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_regular_templates
"""Auto-ported test: TestTemplate::test_regular_templates (CPython 3.12 oracle)."""


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
s = Template('$who likes to eat a bag of $what worth $$100')

assert s.substitute(dict(who='tim', what='ham')) == 'tim likes to eat a bag of ham worth $100'

try:
    s.substitute(dict(who='tim'))
    raise AssertionError('expected KeyError')
except KeyError:
    pass

try:
    Template.substitute()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestTemplate::test_regular_templates: ok")
