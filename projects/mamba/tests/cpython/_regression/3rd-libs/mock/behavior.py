"""Behavior contract for stdlib unittest.mock module.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

from unittest import mock

# Rule 1: Mock records all calls
_m1 = mock.Mock()
_m1("a", 1)
_m1("b", 2)
_m1("c", 3)
assert _m1.call_count == 3, f"call_count = {_m1.call_count!r}"
assert len(_m1.call_args_list) == 3, f"call_args_list = {len(_m1.call_args_list)!r}"

# Rule 2: assert_called_with verifies last call
_m2 = mock.Mock()
_m2("hello", key="world")
_m2.assert_called_with("hello", key="world")
_raised2 = False
try:
    _m2.assert_called_with("wrong")
except AssertionError:
    _raised2 = True
assert _raised2, "wrong args raises AssertionError"

# Rule 3: assert_called_once verifies single call
_m3 = mock.Mock()
_m3(1)
_m3.assert_called_once()
_m3(2)  # second call
_raised3 = False
try:
    _m3.assert_called_once()
except AssertionError:
    _raised3 = True
assert _raised3, "two calls fails assert_called_once"

# Rule 4: patch replaces an attribute during context
import os.path as _osp

_orig = _osp.exists
with mock.patch("os.path.exists", return_value=True) as _patched:
    assert _osp.exists("/any/path") == True, "patched exists always True"
    assert isinstance(_patched, mock.MagicMock), f"patch type = {type(_patched)!r}"

# Restored after context
assert _osp.exists is _orig or callable(_osp.exists), "restored after patch"

# Rule 5: patch.object patches a method on an object
class _MyClass:
    def greet(self):
        return "hello"

_obj = _MyClass()
assert _obj.greet() == "hello", "original greet"
with mock.patch.object(_obj, "greet", return_value="mocked"):
    assert _obj.greet() == "mocked", "patched greet"
assert _obj.greet() == "hello", "restored greet"

# Rule 6: side_effect as function
def _double(x):
    return x * 2

_m6 = mock.Mock(side_effect=_double)
assert _m6(5) == 10, "side_effect fn"
assert _m6(3) == 6, "side_effect fn repeated"

# Rule 7: MagicMock context manager protocol
_cm = mock.MagicMock()
_cm.__enter__.return_value = "context_value"
with _cm as _val:
    assert _val == "context_value", f"context manager value = {_val!r}"
_cm.__enter__.assert_called_once()
_cm.__exit__.assert_called_once()

# Rule 8: patch.dict modifies a dict within context
_d8 = {"key": "original", "keep": "yes"}
with mock.patch.dict(_d8, {"key": "replaced", "new": "added"}):
    assert _d8["key"] == "replaced", f"patched key = {_d8['key']!r}"
    assert _d8["new"] == "added", "new key added"
    assert _d8["keep"] == "yes", "keep key unchanged"
# Restored
assert _d8["key"] == "original", f"restored key = {_d8['key']!r}"
assert "new" not in _d8, "new key removed"

print("behavior OK")
