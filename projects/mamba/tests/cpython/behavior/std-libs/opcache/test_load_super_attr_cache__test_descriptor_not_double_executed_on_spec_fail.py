# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_load_super_attr_cache__test_descriptor_not_double_executed_on_spec_fail"
# subject = "cpython.test_opcache.TestLoadSuperAttrCache.test_descriptor_not_double_executed_on_spec_fail"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcache.py::TestLoadSuperAttrCache::test_descriptor_not_double_executed_on_spec_fail
"""Auto-ported test: TestLoadSuperAttrCache::test_descriptor_not_double_executed_on_spec_fail (CPython 3.12 oracle)."""


import unittest


# --- test body ---
calls = []

class Descriptor:

    def __get__(self, instance, owner):
        calls.append((instance, owner))
        return lambda: 1

class C:
    d = Descriptor()

class D(C):

    def f(self):
        return super().d()
d = D()

assert d.f() == 1
calls.clear()

assert d.f() == 1

assert calls == [(d, D)]
print("TestLoadSuperAttrCache::test_descriptor_not_double_executed_on_spec_fail: ok")
