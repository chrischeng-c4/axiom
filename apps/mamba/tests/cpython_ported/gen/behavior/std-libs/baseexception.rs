use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/baseexception/exception_class_tests__test_builtins_new_style.py`.
#[test]
fn test_gen_behavior_std_libs_baseexception_exception_class_tests__test_builtins_new_style() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "baseexception"
# dimension = "behavior"
# case = "exception_class_tests__test_builtins_new_style"
# subject = "cpython.test_baseexception.ExceptionClassTests.test_builtins_new_style"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_baseexception.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_baseexception.py::ExceptionClassTests::test_builtins_new_style
"""Auto-ported test: ExceptionClassTests::test_builtins_new_style (CPython 3.12 oracle)."""


import unittest
import builtins
import os
from platform import system as platform_system


# --- test body ---
interface_tests = ('length', 'args', 'str', 'repr')

assert issubclass(Exception, object)
print("ExceptionClassTests::test_builtins_new_style: ok")
"###);
    assert_output(&out, r###"ExceptionClassTests::test_builtins_new_style: ok
"###);
}
