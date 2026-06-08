# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_load_method_cache__test_metaclass_descriptor_shadows_class_attribute"
# subject = "cpython.test_opcache.TestLoadMethodCache.test_metaclass_descriptor_shadows_class_attribute"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_opcache.py::TestLoadMethodCache::test_metaclass_descriptor_shadows_class_attribute
"""Auto-ported test: TestLoadMethodCache::test_metaclass_descriptor_shadows_class_attribute (CPython 3.12 oracle)."""


import unittest


# --- test body ---
class Metaclass(type):

    @property
    def attribute(self):
        return lambda: True

class Class(metaclass=Metaclass):

    def attribute():
        return False

def f():
    return Class.attribute()
for _ in range(1025):

    assert f()
print("TestLoadMethodCache::test_metaclass_descriptor_shadows_class_attribute: ok")
