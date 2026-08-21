# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_load_attr_cache__test_metaclass_descriptor_shadows_class_attribute"
# subject = "cpython.test_opcache.TestLoadAttrCache.test_metaclass_descriptor_shadows_class_attribute"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_opcache.py::TestLoadAttrCache::test_metaclass_descriptor_shadows_class_attribute
"""Auto-ported test: TestLoadAttrCache::test_metaclass_descriptor_shadows_class_attribute (CPython 3.12 oracle)."""


import unittest


# --- test body ---
class Metaclass(type):

    @property
    def attribute(self):
        return True

class Class(metaclass=Metaclass):
    attribute = False

def f():
    return Class.attribute
for _ in range(1025):

    assert f()
print("TestLoadAttrCache::test_metaclass_descriptor_shadows_class_attribute: ok")
