# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_load_method_cache__test_metaclass_swap"
# subject = "cpython.test_opcache.TestLoadMethodCache.test_metaclass_swap"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcache.py::TestLoadMethodCache::test_metaclass_swap
"""Auto-ported test: TestLoadMethodCache::test_metaclass_swap (CPython 3.12 oracle)."""


import unittest


# --- test body ---
class OldMetaclass(type):

    @property
    def attribute(self):
        return lambda: True

class NewMetaclass(type):

    @property
    def attribute(self):
        return lambda: False

class Class(metaclass=OldMetaclass):
    pass

def f():
    return Class.attribute()
for _ in range(1025):

    assert f()
Class.__class__ = NewMetaclass
for _ in range(1025):

    assert not f()
print("TestLoadMethodCache::test_metaclass_swap: ok")
