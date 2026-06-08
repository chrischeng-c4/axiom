"""Surface contract for stdlib unittest.mock module.

# type-regime: monomorphic

Probes: unittest.mock.Mock, MagicMock, patch, call, ANY, sentinel,
PropertyMock, AsyncMock, MagicMock.__class_getitem__,
Mock.return_value, Mock.side_effect, Mock.called, Mock.call_count,
Mock.assert_called_with, Mock.assert_called_once, patch.object,
patch.dict.
CPython 3.12 is the oracle.
"""

from unittest import mock

# Core classes
assert hasattr(mock, "Mock"), "Mock"
assert hasattr(mock, "MagicMock"), "MagicMock"
assert hasattr(mock, "AsyncMock"), "AsyncMock"
assert hasattr(mock, "PropertyMock"), "PropertyMock"
assert hasattr(mock, "NonCallableMock"), "NonCallableMock"

# patch
assert hasattr(mock, "patch"), "patch"
assert hasattr(mock.patch, "object"), "patch.object"
assert hasattr(mock.patch, "dict"), "patch.dict"
assert hasattr(mock.patch, "multiple"), "patch.multiple"
assert hasattr(mock.patch, "stopall"), "patch.stopall"

# call and ANY sentinels
assert hasattr(mock, "call"), "call"
assert hasattr(mock, "ANY"), "ANY"
assert hasattr(mock, "sentinel"), "sentinel"

# Mock construction
_m = mock.Mock()
assert isinstance(_m, mock.Mock), f"Mock type = {type(_m)!r}"
assert hasattr(_m, "return_value"), "return_value"
assert hasattr(_m, "side_effect"), "side_effect"
assert hasattr(_m, "called"), "called"
assert hasattr(_m, "call_count"), "call_count"
assert hasattr(_m, "call_args"), "call_args"
assert hasattr(_m, "call_args_list"), "call_args_list"

# Not called initially
assert not _m.called, "not called initially"
assert _m.call_count == 0, f"call_count = {_m.call_count!r}"

# Call it
_m(1, 2, key="val")
assert _m.called, "called after invocation"
assert _m.call_count == 1, f"call_count = {_m.call_count!r}"
assert _m.call_args == mock.call(1, 2, key="val"), f"call_args = {_m.call_args!r}"

# return_value
_m.return_value = 42
assert _m() == 42, f"return_value = {_m()!r}"

# side_effect
_m2 = mock.Mock(side_effect=[10, 20, ValueError("err")])
assert _m2() == 10, "side_effect first"
assert _m2() == 20, "side_effect second"
_raised = False
try:
    _m2()
except ValueError:
    _raised = True
assert _raised, "side_effect exception"

# MagicMock supports magic methods
_mm = mock.MagicMock()
assert len(_mm) == 0, "MagicMock __len__ default"
_mm.__len__.return_value = 5
assert len(_mm) == 5, "MagicMock __len__ override"

# ANY equals anything
assert mock.ANY == 42, "ANY == 42"
assert mock.ANY == "hello", "ANY == str"
assert mock.ANY == [], "ANY == list"
assert mock.ANY == mock.ANY, "ANY == ANY"

print("surface OK")
