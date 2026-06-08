# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "behavior"
# case = "test_template__test_braced_override_safe"
# subject = "cpython.test_string.TestTemplate.test_braced_override_safe"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_string.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_string.py::TestTemplate::test_braced_override_safe
"""Auto-ported test: TestTemplate::test_braced_override_safe (CPython 3.12 oracle)."""


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
class MyTemplate(Template):
    pattern = '\n            \\$(?:\n              (?P<escaped>$)                     |\n              (?P<named>[_a-z][_a-z0-9]*)        |\n              @@(?P<braced>[_a-z][_a-z0-9]*)@@   |\n              (?P<invalid>)                      |\n           )\n           '
tmpl = 'PyCon in $@@location@@'
t = MyTemplate(tmpl)

assert t.safe_substitute() == tmpl
val = t.safe_substitute({'location': 'Cleveland'})

assert val == 'PyCon in Cleveland'
print("TestTemplate::test_braced_override_safe: ok")
