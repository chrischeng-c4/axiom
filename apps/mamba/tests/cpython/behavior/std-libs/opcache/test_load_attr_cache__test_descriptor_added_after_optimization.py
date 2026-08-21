# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_load_attr_cache__test_descriptor_added_after_optimization"
# subject = "cpython.test_opcache.TestLoadAttrCache.test_descriptor_added_after_optimization"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcache.py::TestLoadAttrCache::test_descriptor_added_after_optimization
"""Auto-ported test: TestLoadAttrCache::test_descriptor_added_after_optimization (CPython 3.12 oracle)."""


import unittest


# --- test body ---
class Descriptor:
    pass

class C:

    def __init__(self):
        self.x = 1
    x = Descriptor()

def f(o):
    return o.x
o = C()
for i in range(1025):
    assert f(o) == 1
Descriptor.__get__ = lambda self, instance, value: 2
Descriptor.__set__ = lambda *args: None

assert f(o) == 2
print("TestLoadAttrCache::test_descriptor_added_after_optimization: ok")
