use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/unittest_mock/assert_called_never_called_raises.py`.
#[test]
fn test_gen_errors_std_libs_unittest_mock_assert_called_never_called_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "assert_called_never_called_raises"
# subject = "unittest.mock.Mock.assert_called"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.assert_called: assert_called_never_called_raises (errors)."""
import unittest.mock

_raised = False
try:
    unittest.mock.MagicMock().assert_called()
except AssertionError:
    _raised = True
assert _raised, "assert_called_never_called_raises: expected AssertionError"
print("assert_called_never_called_raises OK")
"###);
    assert_output(&out, r###"assert_called_never_called_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unittest_mock/assert_called_once_when_called_twice_raises.py`.
#[test]
fn test_gen_errors_std_libs_unittest_mock_assert_called_once_when_called_twice_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "assert_called_once_when_called_twice_raises"
# subject = "unittest.mock.Mock.assert_called_once"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.assert_called_once: a mock called twice fails assert_called_once() with AssertionError whose message reports it was Called 2 times"""
from unittest.mock import MagicMock

m = MagicMock()
m()
m()
_raised = False
msg = ""
try:
    m.assert_called_once()
except AssertionError as e:
    _raised = True
    msg = str(e)
assert _raised, "expected AssertionError when called twice"
assert "2 times" in msg, msg
print("assert_called_once_when_called_twice_raises OK")
"###);
    assert_output(&out, r###"assert_called_once_when_called_twice_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unittest_mock/assert_called_once_with_no_call_raises.py`.
#[test]
fn test_gen_errors_std_libs_unittest_mock_assert_called_once_with_no_call_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "assert_called_once_with_no_call_raises"
# subject = "unittest.mock.Mock.assert_called_once_with"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.assert_called_once_with: assert_called_once_with_no_call_raises (errors)."""
import unittest.mock

_raised = False
try:
    unittest.mock.MagicMock().assert_called_once_with(42)
except AssertionError:
    _raised = True
assert _raised, "assert_called_once_with_no_call_raises: expected AssertionError"
print("assert_called_once_with_no_call_raises OK")
"###);
    assert_output(&out, r###"assert_called_once_with_no_call_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unittest_mock/assert_called_with_wrong_args_raises.py`.
#[test]
fn test_gen_errors_std_libs_unittest_mock_assert_called_with_wrong_args_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "assert_called_with_wrong_args_raises"
# subject = "unittest.mock.Mock.assert_called_with"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.assert_called_with: after one no-arg call, assert_called_with(42) raises AssertionError because the recorded call args do not match"""
from unittest.mock import MagicMock

m = MagicMock()
m()  # recorded call has no arguments
_raised = False
try:
    m.assert_called_with(42)
except AssertionError:
    _raised = True
assert _raised, "expected AssertionError for mismatched call args"
print("assert_called_with_wrong_args_raises OK")
"###);
    assert_output(&out, r###"assert_called_with_wrong_args_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unittest_mock/assert_not_called_after_call_raises.py`.
#[test]
fn test_gen_errors_std_libs_unittest_mock_assert_not_called_after_call_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "assert_not_called_after_call_raises"
# subject = "unittest.mock.Mock.assert_not_called"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.assert_not_called: after a call, assert_not_called() raises AssertionError"""
from unittest.mock import MagicMock

m = MagicMock()
m()
_raised = False
try:
    m.assert_not_called()
except AssertionError:
    _raised = True
assert _raised, "expected AssertionError after a call"
print("assert_not_called_after_call_raises OK")
"###);
    assert_output(&out, r###"assert_not_called_after_call_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unittest_mock/create_autospec_wrong_signature_raises.py`.
#[test]
fn test_gen_errors_std_libs_unittest_mock_create_autospec_wrong_signature_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "create_autospec_wrong_signature_raises"
# subject = "unittest.mock.create_autospec"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.create_autospec: create_autospec(func) enforces the wrapped signature: calling it with a missing required argument raises TypeError"""
from unittest.mock import create_autospec


def f(a, b):
    return a + b


af = create_autospec(f)
af(1, 2)  # valid call is accepted
_raised = False
try:
    af(1)  # missing required argument b
except TypeError:
    _raised = True
assert _raised, "autospec must enforce the wrapped signature"
print("create_autospec_wrong_signature_raises OK")
"###);
    assert_output(&out, r###"create_autospec_wrong_signature_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unittest_mock/patch_object_missing_attr_raises.py`.
#[test]
fn test_gen_errors_std_libs_unittest_mock_patch_object_missing_attr_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "patch_object_missing_attr_raises"
# subject = "unittest.mock.patch.object"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testpatch.py"
# status = "filled"
# ///
"""unittest.mock.patch.object: patch.object(cls, 'no_such_method') as a context manager raises AttributeError because the target attribute does not exist"""
from unittest.mock import patch


class T:
    def method(self) -> int:
        return 1


_raised = False
try:
    with patch.object(T, "no_such_method"):
        pass
except AttributeError:
    _raised = True
assert _raised, "expected AttributeError patching a missing attribute"
print("patch_object_missing_attr_raises OK")
"###);
    assert_output(&out, r###"patch_object_missing_attr_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unittest_mock/plain_mock_len_raises.py`.
#[test]
fn test_gen_errors_std_libs_unittest_mock_plain_mock_len_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "plain_mock_len_raises"
# subject = "unittest.mock.Mock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock: plain_mock_len_raises (errors)."""
import unittest.mock

_raised = False
try:
    len(unittest.mock.Mock())
except TypeError:
    _raised = True
assert _raised, "plain_mock_len_raises: expected TypeError"
print("plain_mock_len_raises OK")
"###);
    assert_output(&out, r###"plain_mock_len_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unittest_mock/sealed_mock_new_attr_raises.py`.
#[test]
fn test_gen_errors_std_libs_unittest_mock_sealed_mock_new_attr_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "sealed_mock_new_attr_raises"
# subject = "unittest.mock.seal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testsealable.py"
# status = "filled"
# ///
"""unittest.mock.seal: after seal(mock), touching an unconfigured child attribute raises AttributeError"""
from unittest.mock import MagicMock, seal

m = MagicMock()
m.configured.return_value = 1
seal(m)
assert m.configured() == 1  # configured child still works
_raised = False
try:
    m.unconfigured.deep
except AttributeError:
    _raised = True
assert _raised, "a sealed mock must block a new child attribute"
print("sealed_mock_new_attr_raises OK")
"###);
    assert_output(&out, r###"sealed_mock_new_attr_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unittest_mock/side_effect_exception_propagates.py`.
#[test]
fn test_gen_errors_std_libs_unittest_mock_side_effect_exception_propagates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "side_effect_exception_propagates"
# subject = "unittest.mock.Mock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock: a mock whose side_effect is an exception instance re-raises that exception when called"""
from unittest.mock import MagicMock

m = MagicMock(side_effect=ValueError("boom"))
_raised = False
try:
    m()
except ValueError as e:
    _raised = True
    assert str(e) == "boom"
assert _raised, "an exception side_effect must be re-raised"
print("side_effect_exception_propagates OK")
"###);
    assert_output(&out, r###"side_effect_exception_propagates OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/unittest_mock/spec_unknown_attr_raises.py`.
#[test]
fn test_gen_errors_std_libs_unittest_mock_spec_unknown_attr_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "errors"
# case = "spec_unknown_attr_raises"
# subject = "unittest.mock.Mock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock: a Mock(spec=cls) rejects access to an attribute the spec class does not define, raising AttributeError"""
from unittest.mock import MagicMock


class T:
    def method(self) -> int:
        return 1


m = MagicMock(spec=T)
_raised = False
try:
    m.no_such_method
except AttributeError:
    _raised = True
assert _raised, "spec must reject an undeclared attribute"
print("spec_unknown_attr_raises OK")
"###);
    assert_output(&out, r###"spec_unknown_attr_raises OK
"###);
}
