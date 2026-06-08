# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_call_cache__test_too_many_defaults_1"
# subject = "cpython.test_opcache.TestCallCache.test_too_many_defaults_1"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_opcache.py::TestCallCache::test_too_many_defaults_1
"""Auto-ported test: TestCallCache::test_too_many_defaults_1 (CPython 3.12 oracle)."""


import unittest


# --- test body ---
def f(x):
    pass
f.__defaults__ = (None, None)
for _ in range(1025):
    f(None)
    f()
print("TestCallCache::test_too_many_defaults_1: ok")
