# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_load_attr_cache__test_metaclass_getattribute"
# subject = "cpython.test_opcache.TestLoadAttrCache.test_metaclass_getattribute"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcache.py::TestLoadAttrCache::test_metaclass_getattribute
"""Auto-ported test: TestLoadAttrCache::test_metaclass_getattribute (CPython 3.12 oracle)."""


import unittest


# --- test body ---
class Metaclass(type):

    def __getattribute__(self, name):
        return True

class Class(metaclass=Metaclass):
    attribute = False

def f():
    return Class.attribute
for _ in range(1025):

    assert f()
print("TestLoadAttrCache::test_metaclass_getattribute: ok")
