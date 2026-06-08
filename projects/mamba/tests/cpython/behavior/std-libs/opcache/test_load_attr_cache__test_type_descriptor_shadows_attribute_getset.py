# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_load_attr_cache__test_type_descriptor_shadows_attribute_getset"
# subject = "cpython.test_opcache.TestLoadAttrCache.test_type_descriptor_shadows_attribute_getset"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_opcache.py::TestLoadAttrCache::test_type_descriptor_shadows_attribute_getset
"""Auto-ported test: TestLoadAttrCache::test_type_descriptor_shadows_attribute_getset (CPython 3.12 oracle)."""


import unittest


# --- test body ---
class Class:
    __name__ = 'Spam'

def f():
    return Class.__name__
for _ in range(1025):

    assert f() == 'Class'
print("TestLoadAttrCache::test_type_descriptor_shadows_attribute_getset: ok")
