# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_load_attr_cache__test_store_shadowing_slot_should_raise_type_error"
# subject = "cpython.test_opcache.TestLoadAttrCache.test_store_shadowing_slot_should_raise_type_error"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcache.py::TestLoadAttrCache::test_store_shadowing_slot_should_raise_type_error
"""Auto-ported test: TestLoadAttrCache::test_store_shadowing_slot_should_raise_type_error (CPython 3.12 oracle)."""


import unittest


# --- test body ---
class Class:
    __slots__ = ('slot',)

class Sneaky:
    __slots__ = ('shadowed',)
    shadowing = Class.slot

def f(o):
    o.shadowing = 42
o = Sneaky()
for _ in range(1025):
    try:
        f(o)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("TestLoadAttrCache::test_store_shadowing_slot_should_raise_type_error: ok")
