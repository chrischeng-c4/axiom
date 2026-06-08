# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_load_attr_cache__test_metaclass_descriptor_added_after_optimization"
# subject = "cpython.test_opcache.TestLoadAttrCache.test_metaclass_descriptor_added_after_optimization"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcache.py::TestLoadAttrCache::test_metaclass_descriptor_added_after_optimization
"""Auto-ported test: TestLoadAttrCache::test_metaclass_descriptor_added_after_optimization (CPython 3.12 oracle)."""


import unittest


# --- test body ---
class Descriptor:
    pass

class Metaclass(type):
    attribute = Descriptor()

class Class(metaclass=Metaclass):
    attribute = True

def __get__(self, instance, owner):
    return False

def __set__(self, instance, value):
    return None

def f():
    return Class.attribute
for _ in range(1025):

    assert f()
Descriptor.__get__ = __get__
Descriptor.__set__ = __set__
for _ in range(1025):

    assert not f()
print("TestLoadAttrCache::test_metaclass_descriptor_added_after_optimization: ok")
