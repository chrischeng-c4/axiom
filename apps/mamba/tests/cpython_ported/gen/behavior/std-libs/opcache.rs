use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/opcache/test_call_cache__test_too_many_defaults_0.py`.
#[test]
fn test_gen_behavior_std_libs_opcache_test_call_cache__test_too_many_defaults_0() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"TestCallCache::test_too_many_defaults_0: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/opcache/test_call_cache__test_too_many_defaults_1.py`.
#[test]
fn test_gen_behavior_std_libs_opcache_test_call_cache__test_too_many_defaults_1() {
    let out = jit_capture(r###"# /// script
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
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
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
"###);
    assert_output(&out, r###"TestCallCache::test_too_many_defaults_1: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/opcache/test_call_cache__test_too_many_defaults_2.py`.
#[test]
fn test_gen_behavior_std_libs_opcache_test_call_cache__test_too_many_defaults_2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_call_cache__test_too_many_defaults_2"
# subject = "cpython.test_opcache.TestCallCache.test_too_many_defaults_2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_opcache.py::TestCallCache::test_too_many_defaults_2
"""Auto-ported test: TestCallCache::test_too_many_defaults_2 (CPython 3.12 oracle)."""


import unittest


# --- test body ---
def f(x, y):
    pass
f.__defaults__ = (None, None, None)
for _ in range(1025):
    f(None, None)
    f(None)
    f()
print("TestCallCache::test_too_many_defaults_2: ok")
"###);
    assert_output(&out, r###"TestCallCache::test_too_many_defaults_2: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/opcache/test_load_attr_cache__test_type_descriptor_shadows_attribute_getset.py`.
#[test]
fn test_gen_behavior_std_libs_opcache_test_load_attr_cache__test_type_descriptor_shadows_attribute_getset() {
    let out = jit_capture(r###"# /// script
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
"###);
    assert_output(&out, r###"TestLoadAttrCache::test_type_descriptor_shadows_attribute_getset: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/opcache/test_load_method_cache__test_descriptor_added_after_optimization.py`.
#[test]
fn test_gen_behavior_std_libs_opcache_test_load_method_cache__test_descriptor_added_after_optimization() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "opcache"
# dimension = "behavior"
# case = "test_load_method_cache__test_descriptor_added_after_optimization"
# subject = "cpython.test_opcache.TestLoadMethodCache.test_descriptor_added_after_optimization"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_opcache.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_opcache.py::TestLoadMethodCache::test_descriptor_added_after_optimization
"""Auto-ported test: TestLoadMethodCache::test_descriptor_added_after_optimization (CPython 3.12 oracle)."""


import unittest


# --- test body ---
class Descriptor:
    pass

class Class:
    attribute = Descriptor()

def __get__(self, instance, owner):
    return lambda: False

def __set__(self, instance, value):
    return None

def attribute():
    return True
instance = Class()
instance.attribute = attribute

def f():
    return instance.attribute()
for _ in range(1025):

    assert f()
Descriptor.__get__ = __get__
Descriptor.__set__ = __set__
for _ in range(1025):

    assert not f()
print("TestLoadMethodCache::test_descriptor_added_after_optimization: ok")
"###);
    assert_output(&out, r###"TestLoadMethodCache::test_descriptor_added_after_optimization: ok
"###);
}
