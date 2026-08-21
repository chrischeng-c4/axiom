# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_load_method_cache__test_type_descriptor_shadows_attribute_method"
# subject = "cpython.test_opcache.TestLoadMethodCache.test_type_descriptor_shadows_attribute_method"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_opcache.py::TestLoadMethodCache::test_type_descriptor_shadows_attribute_method
"""Auto-ported test: TestLoadMethodCache::test_type_descriptor_shadows_attribute_method (CPython 3.12 oracle)."""


import unittest


# --- test body ---
class Class:

    def mro():
        return ['Spam', 'eggs']

def f():
    return Class.mro()
for _ in range(1025):

    assert f() == ['Spam', 'eggs']
print("TestLoadMethodCache::test_type_descriptor_shadows_attribute_method: ok")
