use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/unittest/assertequal_mismatch_raises.py`.
#[test]
fn test_gen_errors_std_libs_unittest_assertequal_mismatch_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "errors"
# case = "assertequal_mismatch_raises"
# subject = "unittest.TestCase.assertEqual"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase.assertEqual: calling assertEqual(1, 2) on a TestCase instance raises the failureException (AssertionError) outside a runner"""
import unittest


class Sample(unittest.TestCase):
    def runTest(self):
        pass


tc = Sample()
_raised = False
try:
    tc.assertEqual(1, 2)
except AssertionError:
    _raised = True
assert _raised, "assertequal_mismatch_raises: expected AssertionError"
print("assertequal_mismatch_raises OK")
"###);
    assert_output(&out, r###"assertequal_mismatch_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unittest/assertraises_no_exception_raises.py`.
#[test]
fn test_gen_errors_std_libs_unittest_assertraises_no_exception_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "errors"
# case = "assertraises_no_exception_raises"
# subject = "unittest.TestCase.assertRaises"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase.assertRaises: an assertRaises(ValueError) block whose body raises nothing raises AssertionError on context exit"""
import unittest


class Sample(unittest.TestCase):
    def runTest(self):
        pass


tc = Sample()
_raised = False
try:
    with tc.assertRaises(ValueError):
        pass
except AssertionError:
    _raised = True
assert _raised, "assertraises_no_exception_raises: expected AssertionError"
print("assertraises_no_exception_raises OK")
"###);
    assert_output(&out, r###"assertraises_no_exception_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unittest/bare_testcase_run_missing_method_raises.py`.
#[test]
fn test_gen_errors_std_libs_unittest_bare_testcase_run_missing_method_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "errors"
# case = "bare_testcase_run_missing_method_raises"
# subject = "unittest.TestCase.run"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.TestCase.run: running a bare unittest.TestCase() with no selected test method raises AttributeError (no _testMethodName-resolvable method)"""
import unittest

bare = unittest.TestCase()
_raised = False
try:
    bare.run()
except AttributeError:
    _raised = True
assert _raised, "bare_testcase_run_missing_method_raises: expected AttributeError"
print("bare_testcase_run_missing_method_raises OK")
"###);
    assert_output(&out, r###"bare_testcase_run_missing_method_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unittest/named_ctor_missing_method_raises.py`.
#[test]
fn test_gen_errors_std_libs_unittest_named_ctor_missing_method_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "errors"
# case = "named_ctor_missing_method_raises"
# subject = "unittest.FunctionTestCase"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/"
# status = "filled"
# ///
"""unittest.FunctionTestCase: constructing a TestCase with a method name that is not an attribute of the class raises ValueError"""
import unittest


class Sample(unittest.TestCase):
    def test_a(self):
        pass

    def runTest(self):
        pass


_raised = False
try:
    Sample("does_not_exist")
except ValueError:
    _raised = True
assert _raised, "named_ctor_missing_method_raises: expected ValueError"
print("named_ctor_missing_method_raises OK")
"###);
    assert_output(&out, r###"named_ctor_missing_method_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unittest/skiptest_raise_is_skiptest.py`.
#[test]
fn test_gen_errors_std_libs_unittest_skiptest_raise_is_skiptest() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest"
# dimension = "errors"
# case = "skiptest_raise_is_skiptest"
# subject = "unittest.SkipTest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""unittest.SkipTest: skiptest_raise_is_skiptest (errors)."""
import unittest

_raised = False
try:
    raise unittest.SkipTest('skipping')
except unittest.SkipTest:
    _raised = True
assert _raised, "skiptest_raise_is_skiptest: expected unittest.SkipTest"
print("skiptest_raise_is_skiptest OK")
"###);
    assert_output(&out, r###"skiptest_raise_is_skiptest OK
"###);
}
