# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_load_method_cache__test_descriptor_added_after_optimization"
# subject = "cpython.test_opcache.TestLoadMethodCache.test_descriptor_added_after_optimization"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_opcache.py::TestLoadMethodCache::test_descriptor_added_after_optimization
"""Auto-ported test: TestLoadMethodCache::test_descriptor_added_after_optimization (CPython 3.12 oracle)."""


import unittest


# --- test body ---
class Descriptor:
    pass

class Class:
    attribute = Descriptor()

def __get__(self, instance, owner):
    return lambda: False

def __set__(self, instance, value):
    return None

def attribute():
    return True
instance = Class()
instance.attribute = attribute

def f():
    return instance.attribute()
for _ in range(1025):

    assert f()
Descriptor.__get__ = __get__
Descriptor.__set__ = __set__
for _ in range(1025):

    assert not f()
print("TestLoadMethodCache::test_descriptor_added_after_optimization: ok")
