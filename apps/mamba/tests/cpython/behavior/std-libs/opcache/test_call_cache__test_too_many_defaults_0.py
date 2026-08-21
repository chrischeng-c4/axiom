# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_call_cache__test_too_many_defaults_0"
# subject = "cpython.test_opcache.TestCallCache.test_too_many_defaults_0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_opcache.py::TestCallCache::test_too_many_defaults_0
"""Auto-ported test: TestCallCache::test_too_many_defaults_0 (CPython 3.12 oracle)."""


import unittest


# --- test body ---
def f():
    pass
f.__defaults__ = (None,)
for _ in range(1025):
    f()
print("TestCallCache::test_too_many_defaults_0: ok")
