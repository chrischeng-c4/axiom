# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_load_method_cache__test_type_descriptor_shadows_attribute_member"
# subject = "cpython.test_opcache.TestLoadMethodCache.test_type_descriptor_shadows_attribute_member"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcache.py::TestLoadMethodCache::test_type_descriptor_shadows_attribute_member
"""Auto-ported test: TestLoadMethodCache::test_type_descriptor_shadows_attribute_member (CPython 3.12 oracle)."""


import unittest


# --- test body ---
class Class:

    def __base__():
        return 'Spam'

def f():
    return Class.__base__()
for _ in range(1025):

    assert f() != 'Spam'
print("TestLoadMethodCache::test_type_descriptor_shadows_attribute_member: ok")
