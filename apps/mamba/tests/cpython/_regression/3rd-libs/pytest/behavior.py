"""Behavior contract for third-party pytest package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import pytest  # type: ignore[import]

# Rule 1: pytest.raises catches specified exception type
_caught1 = False
with pytest.raises(ValueError):
    _caught1 = True
    raise ValueError("expected")
assert _caught1, "code inside raises executed"

# Rule 2: pytest.raises with match= checks message pattern
with pytest.raises(TypeError, match="unsupported"):
    raise TypeError("unsupported operand type(s)")

# Rule 3: ExceptionInfo has type, value, traceback attributes
with pytest.raises(RuntimeError, match="boom") as _ei3:
    raise RuntimeError("boom")
assert _ei3.type is RuntimeError, f"type = {_ei3.type!r}"
assert isinstance(_ei3.value, RuntimeError), "value is RuntimeError"
assert str(_ei3.value) == "boom", f"message = {str(_ei3.value)!r}"

# Rule 4: pytest.approx compares floats with tolerance
assert 0.1 + 0.2 == pytest.approx(0.3), "float approx"
assert 1.0000001 == pytest.approx(1.0, rel=1e-5), "approx rel"
assert 1.0000001 == pytest.approx(1.0, abs=1e-5), "approx abs"
# Not approx when difference is large
assert 1.1 != pytest.approx(1.0, rel=1e-6), "not approx large diff"

# Rule 5: pytest.approx works with lists and dicts
assert [0.1, 0.2, 0.3] == pytest.approx([0.1, 0.2, 0.3]), "list approx"
assert {"a": 0.1, "b": 0.2} == pytest.approx({"a": 0.1, "b": 0.2}), "dict approx"

# Rule 6: pytest.param carries values and id
_p6 = pytest.param(10, 20, 30, id="triple")
assert hasattr(_p6, "id"), "param has id"
assert _p6.id == "triple", f"param id = {_p6.id!r}"
assert hasattr(_p6, "values"), "param has values"
assert _p6.values == (10, 20, 30), f"param values = {_p6.values!r}"

# Rule 7: pytest.raises is context manager — works without 'with' (raises.excinfo)
_raises7 = pytest.raises(ValueError)
assert hasattr(_raises7, "__enter__"), "raises has __enter__"
assert hasattr(_raises7, "__exit__"), "raises has __exit__"

# Rule 8: pytest.mark has parametrize/skip/xfail/skipif/usefixtures
_marks = ["parametrize", "skip", "xfail", "skipif", "usefixtures"]
for _m in _marks:
    assert hasattr(pytest.mark, _m), f"mark.{_m}"
    _marker = getattr(pytest.mark, _m)
    assert callable(_marker), f"mark.{_m} callable"

print("behavior OK")
