# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_load_method_cache__test_metaclass_del_descriptor_after_optimization"
# subject = "cpython.test_opcache.TestLoadMethodCache.test_metaclass_del_descriptor_after_optimization"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcache.py::TestLoadMethodCache::test_metaclass_del_descriptor_after_optimization
"""Auto-ported test: TestLoadMethodCache::test_metaclass_del_descriptor_after_optimization (CPython 3.12 oracle)."""


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
del Metaclass.attribute
for _ in range(1025):

    assert not f()
print("TestLoadMethodCache::test_metaclass_del_descriptor_after_optimization: ok")
