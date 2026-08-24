use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/subclassinit/test__test_init_subclass_kwargs.py`.
#[test]
fn test_gen_behavior_std_libs_subclassinit_test__test_init_subclass_kwargs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_init_subclass_kwargs"
# subject = "cpython.test_subclassinit.Test.test_init_subclass_kwargs"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_subclassinit.py::Test::test_init_subclass_kwargs
"""Auto-ported test: Test::test_init_subclass_kwargs (CPython 3.12 oracle)."""


import types
import unittest


# --- test body ---
class A:

    def __init_subclass__(cls, **kwargs):
        cls.kwargs = kwargs

class B(A, x=3):
    pass

assert B.kwargs == dict(x=3)
print("Test::test_init_subclass_kwargs: ok")
"###);
    assert_output(&out, r###"Test::test_init_subclass_kwargs: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/subclassinit/test__test_init_subclass_skipped.py`.
#[test]
fn test_gen_behavior_std_libs_subclassinit_test__test_init_subclass_skipped() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_init_subclass_skipped"
# subject = "cpython.test_subclassinit.Test.test_init_subclass_skipped"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_subclassinit.py::Test::test_init_subclass_skipped
"""Auto-ported test: Test::test_init_subclass_skipped (CPython 3.12 oracle)."""


import types
import unittest


# --- test body ---
class BaseWithInit:

    def __init_subclass__(cls, **kwargs):
        super().__init_subclass__(**kwargs)
        cls.initialized = cls

class BaseWithoutInit(BaseWithInit):
    pass

class A(BaseWithoutInit):
    pass

assert A.initialized is A

assert BaseWithoutInit.initialized is BaseWithoutInit
print("Test::test_init_subclass_skipped: ok")
"###);
    assert_output(&out, r###"Test::test_init_subclass_skipped: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/subclassinit/test__test_set_name_lookup.py`.
#[test]
fn test_gen_behavior_std_libs_subclassinit_test__test_set_name_lookup() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subclassinit"
# dimension = "behavior"
# case = "test__test_set_name_lookup"
# subject = "cpython.test_subclassinit.Test.test_set_name_lookup"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subclassinit.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_subclassinit.py::Test::test_set_name_lookup
"""Auto-ported test: Test::test_set_name_lookup (CPython 3.12 oracle)."""


import types
import unittest


# --- test body ---
resolved = []

class NonDescriptor:

    def __getattr__(self, name):
        resolved.append(name)

class A:
    d = NonDescriptor()

assert '__set_name__' not in resolved
print("Test::test_set_name_lookup: ok")
"###);
    assert_output(&out, r###"Test::test_set_name_lookup: ok
"###);
}
