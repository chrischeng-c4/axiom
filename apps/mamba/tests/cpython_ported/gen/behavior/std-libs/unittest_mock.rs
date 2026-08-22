use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/any_matches_any_argument.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_any_matches_any_argument() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "any_matches_any_argument"
# subject = "unittest.mock.ANY"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.ANY: mock.ANY compares equal to any value so assert_called_with(ANY, ANY) accepts any positional arguments"""
from unittest.mock import MagicMock, ANY

m = MagicMock()
m(123, "hi")
m.assert_called_with(ANY, ANY)
assert (ANY == 1) and (ANY == "x") and (ANY == object())
print("any_matches_any_argument OK")
"###);
    assert_output(&out, r###"any_matches_any_argument OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/attribute_access_autocreates_child_mock.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_attribute_access_autocreates_child_mock() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "attribute_access_autocreates_child_mock"
# subject = "unittest.mock.Mock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock: accessing an undefined attribute auto-creates a child mock, the same object is returned on repeated access, and child calls are recorded on the child's call_args_list"""
from unittest.mock import MagicMock, call

m = MagicMock()
child = m.foo
assert m.foo is child  # same child returned on repeated access
m.foo.bar(1)
m.foo.bar(2)
assert m.foo.bar.call_args_list == [call(1), call(2)]
print("attribute_access_autocreates_child_mock OK")
"###);
    assert_output(&out, r###"attribute_access_autocreates_child_mock OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/call_args_args_kwargs_accessors.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_call_args_args_kwargs_accessors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "call_args_args_kwargs_accessors"
# subject = "unittest.mock.Mock.call_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.call_args: call_args exposes .args (a tuple of positionals) and .kwargs (a dict of keywords) for the last call m(1, 2, k=3)"""
from unittest.mock import MagicMock

m = MagicMock()
m(1, 2, k=3)
assert m.call_args.args == (1, 2)
assert m.call_args.kwargs == {"k": 3}
print("call_args_args_kwargs_accessors OK")
"###);
    assert_output(&out, r###"call_args_args_kwargs_accessors OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/call_args_list_records_each_call.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_call_args_list_records_each_call() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "call_args_list_records_each_call"
# subject = "unittest.mock.Mock.call_args_list"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.call_args_list: call_args_list accumulates one call(...) per invocation in order across repeated calls"""
from unittest.mock import MagicMock, call

m = MagicMock()
m(1)
m(2)
m(3)
assert m.call_args_list == [call(1), call(2), call(3)]
print("call_args_list_records_each_call OK")
"###);
    assert_output(&out, r###"call_args_list_records_each_call OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/call_args_none_until_called.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_call_args_none_until_called() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "call_args_none_until_called"
# subject = "unittest.mock.Mock.call_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.call_args: call_args is None before any call and afterwards equals a call(...) capturing the most recent positional and keyword arguments"""
from unittest.mock import MagicMock, call

m = MagicMock()
assert m.call_args is None
m(1, 2, key="value")
assert m.call_args == call(1, 2, key="value")
print("call_args_none_until_called OK")
"###);
    assert_output(&out, r###"call_args_none_until_called OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/call_count_tracks_invocations.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_call_count_tracks_invocations() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "call_count_tracks_invocations"
# subject = "unittest.mock.Mock.call_count"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.call_count: call_count starts at 0 and increments by one per call; .called flips to True after the first call"""
from unittest.mock import MagicMock

m = MagicMock()
assert m.call_count == 0
assert m.called is False
m()
assert m.call_count == 1
assert m.called is True
m()
m()
assert m.call_count == 3
print("call_count_tracks_invocations OK")
"###);
    assert_output(&out, r###"call_count_tracks_invocations OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/call_equality_against_recorded.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_call_equality_against_recorded() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "call_equality_against_recorded"
# subject = "unittest.mock.call"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.call: a freshly constructed call(1, 2, k=3) compares equal to the mock's recorded call_args for the same invocation"""
from unittest.mock import MagicMock, call

m = MagicMock()
m(1, 2, k=3)
assert call(1, 2, k=3) == m.call_args
print("call_equality_against_recorded OK")
"###);
    assert_output(&out, r###"call_equality_against_recorded OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/child_return_value_configuration.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_child_return_value_configuration() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "child_return_value_configuration"
# subject = "unittest.mock.Mock.return_value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.return_value: configuring a deep child's return_value (m.child.method.return_value) makes the corresponding nested call return that value"""
from unittest.mock import MagicMock

m = MagicMock()
m.child.method.return_value = "z"
assert m.child.method() == "z"
print("child_return_value_configuration OK")
"###);
    assert_output(&out, r###"child_return_value_configuration OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/magicmock_context_manager.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_magicmock_context_manager() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "magicmock_context_manager"
# subject = "unittest.mock.MagicMock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testwith.py"
# status = "filled"
# ///
"""unittest.mock.MagicMock: MagicMock works as a context manager: configuring __enter__.return_value makes `with mock as x` bind that value"""
from unittest.mock import MagicMock

m = MagicMock()
m.__enter__.return_value = "ctx"
with m as c:
    assert c == "ctx"
print("magicmock_context_manager OK")
"###);
    assert_output(&out, r###"magicmock_context_manager OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/magicmock_supports_dunder_len.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_magicmock_supports_dunder_len() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "magicmock_supports_dunder_len"
# subject = "unittest.mock.MagicMock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmagicmethods.py"
# status = "filled"
# ///
"""unittest.mock.MagicMock: MagicMock supports magic methods: setting __len__.return_value makes len(mock) return that integer"""
from unittest.mock import MagicMock

m = MagicMock()
m.__len__.return_value = 3
assert len(m) == 3
print("magicmock_supports_dunder_len OK")
"###);
    assert_output(&out, r###"magicmock_supports_dunder_len OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/mock_calls_records_nested_children.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_mock_calls_records_nested_children() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "mock_calls_records_nested_children"
# subject = "unittest.mock.Mock.mock_calls"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.mock_calls: mock_calls on a parent records child-attribute calls as call.child(...) entries in invocation order"""
from unittest.mock import MagicMock, call

parent = MagicMock()
parent.a()
parent.b(2)
assert parent.mock_calls == [call.a(), call.b(2)]
print("mock_calls_records_nested_children OK")
"###);
    assert_output(&out, r###"mock_calls_records_nested_children OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/mock_open_reads_data.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_mock_open_reads_data() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "mock_open_reads_data"
# subject = "unittest.mock.mock_open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.mock_open: mock_open(read_data=...) provides a fake open() whose context-managed file .read() returns the configured data"""
from unittest.mock import mock_open, patch

mo = mock_open(read_data="hello world")
with patch("builtins.open", mo):
    with open("anything") as f:
        data = f.read()
assert data == "hello world"
print("mock_open_reads_data OK")
"###);
    assert_output(&out, r###"mock_open_reads_data OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/patch_context_manager_restores.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_patch_context_manager_restores() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "patch_context_manager_restores"
# subject = "unittest.mock.patch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testpatch.py"
# status = "filled"
# ///
"""unittest.mock.patch: patch as a context manager replaces the target inside the block and restores the original afterwards"""
from unittest.mock import patch, MagicMock
import os

original = os.getpid
with patch("os.getpid", return_value=4321) as m:
    assert os.getpid() == 4321
    assert isinstance(m, MagicMock)
assert os.getpid is original
print("patch_context_manager_restores OK")
"###);
    assert_output(&out, r###"patch_context_manager_restores OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/patch_decorator_injects_mock.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_patch_decorator_injects_mock() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "patch_decorator_injects_mock"
# subject = "unittest.mock.patch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testpatch.py"
# status = "filled"
# ///
"""unittest.mock.patch: patch used as a decorator injects the mock as a parameter and applies the patch for the duration of the call"""
from unittest.mock import patch
import os


@patch("os.getpid", return_value=4321)
def use(mock_getpid):
    return os.getpid()


assert use() == 4321
assert os.getpid() != 4321  # patch lifted after the call
print("patch_decorator_injects_mock OK")
"###);
    assert_output(&out, r###"patch_decorator_injects_mock OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/patch_dict_scopes_mutation.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_patch_dict_scopes_mutation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "patch_dict_scopes_mutation"
# subject = "unittest.mock.patch.dict"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testpatch.py"
# status = "filled"
# ///
"""unittest.mock.patch.dict: patch.dict adds keys to a dict inside the block and reverts the dict to its original contents on exit"""
from unittest.mock import patch

d = {"a": 1}
with patch.dict(d, {"b": 2}):
    assert d == {"a": 1, "b": 2}
assert d == {"a": 1}
print("patch_dict_scopes_mutation OK")
"###);
    assert_output(&out, r###"patch_dict_scopes_mutation OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/patch_start_stop_manual.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_patch_start_stop_manual() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "patch_start_stop_manual"
# subject = "unittest.mock.patch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testpatch.py"
# status = "filled"
# ///
"""unittest.mock.patch: patch().start() applies the patch and the returned mock takes effect until .stop() restores the original"""
from unittest.mock import patch
import os

p = patch("os.getpid", return_value=1)
p.start()
try:
    assert os.getpid() == 1
finally:
    p.stop()
assert os.getpid() != 1
print("patch_start_stop_manual OK")
"###);
    assert_output(&out, r###"patch_start_stop_manual OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/propertymock_intercepts_attribute.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_propertymock_intercepts_attribute() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "propertymock_intercepts_attribute"
# subject = "unittest.mock.PropertyMock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.PropertyMock: a PropertyMock installed on a class intercepts attribute reads, returns its return_value, and records the access"""
from unittest.mock import PropertyMock

pm = PropertyMock(return_value="pv")


class O:
    x = pm


o = O()
assert o.x == "pv"
pm.assert_called_once_with()
print("propertymock_intercepts_attribute OK")
"###);
    assert_output(&out, r###"propertymock_intercepts_attribute OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/reset_mock_clears_call_state.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_reset_mock_clears_call_state() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "reset_mock_clears_call_state"
# subject = "unittest.mock.Mock.reset_mock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.reset_mock: reset_mock() zeroes call_count, empties call_args_list, and sets call_args back to None"""
from unittest.mock import MagicMock

m = MagicMock()
m(1)
m(2)
m.reset_mock()
assert m.call_count == 0
assert m.call_args_list == []
assert m.call_args is None
print("reset_mock_clears_call_state OK")
"###);
    assert_output(&out, r###"reset_mock_clears_call_state OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/return_value_is_returned.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_return_value_is_returned() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "return_value_is_returned"
# subject = "unittest.mock.Mock.return_value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.return_value: a mock configured with return_value returns that exact value on every call"""
from unittest.mock import MagicMock

m = MagicMock(return_value=7)
assert m() == 7
assert m(1, 2) == 7
print("return_value_is_returned OK")
"###);
    assert_output(&out, r###"return_value_is_returned OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/side_effect_callable_computes_return.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_side_effect_callable_computes_return() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "side_effect_callable_computes_return"
# subject = "unittest.mock.Mock.side_effect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.side_effect: a callable side_effect is invoked with the call arguments and its result becomes the mock's return value"""
from unittest.mock import MagicMock

m = MagicMock(side_effect=lambda x: x * 10)
assert m(5) == 50
print("side_effect_callable_computes_return OK")
"###);
    assert_output(&out, r###"side_effect_callable_computes_return OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/side_effect_iterable_yields_in_order.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_side_effect_iterable_yields_in_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "side_effect_iterable_yields_in_order"
# subject = "unittest.mock.Mock.side_effect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock.side_effect: a side_effect set to an iterable returns its elements one per call in order"""
from unittest.mock import MagicMock

m = MagicMock(side_effect=[1, 2, 3])
assert m() == 1
assert m() == 2
assert m() == 3
print("side_effect_iterable_yields_in_order OK")
"###);
    assert_output(&out, r###"side_effect_iterable_yields_in_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/unittest_mock/spec_allows_declared_attributes.py`.
#[test]
fn test_gen_behavior_std_libs_unittest_mock_spec_allows_declared_attributes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "behavior"
# case = "spec_allows_declared_attributes"
# subject = "unittest.mock.Mock"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unittest/testmock/testmock.py"
# status = "filled"
# ///
"""unittest.mock.Mock: a Mock(spec=cls) permits calling methods the spec class declares, recording the call, while restricting undeclared attributes"""
from unittest.mock import MagicMock


class Svc:
    def get(self, k):
        return k


m = MagicMock(spec=Svc)
m.get("a")
assert m.get.called is True
_raised = False
try:
    m.no_such
except AttributeError:
    _raised = True
assert _raised, "spec restricts undeclared attributes"
print("spec_allows_declared_attributes OK")
"###);
    assert_output(&out, r###"spec_allows_declared_attributes OK
"###);
}
