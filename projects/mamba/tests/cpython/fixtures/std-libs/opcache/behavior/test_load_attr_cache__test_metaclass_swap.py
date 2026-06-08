# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_load_attr_cache__test_metaclass_swap"
# subject = "cpython.test_opcache.TestLoadAttrCache.test_metaclass_swap"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcache.py::TestLoadAttrCache::test_metaclass_swap
"""Auto-ported test: TestLoadAttrCache::test_metaclass_swap (CPython 3.12 oracle)."""


import unittest


# --- test body ---
class OldMetaclass(type):

    @property
    def attribute(self):
        return True

class NewMetaclass(type):

    @property
    def attribute(self):
        return False

class Class(metaclass=OldMetaclass):
    pass

def f():
    return Class.attribute
for _ in range(1025):

    assert f()
Class.__class__ = NewMetaclass
for _ in range(1025):

    assert not f()
print("TestLoadAttrCache::test_metaclass_swap: ok")
