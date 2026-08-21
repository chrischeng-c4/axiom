"""Surface contract for third-party pytest package.

# type-regime: monomorphic

Probes: pytest.mark, pytest.fixture, pytest.raises, pytest.approx,
pytest.warns, pytest.skip, pytest.fail, pytest.importorskip,
pytest.param, pytest.main.
CPython 3.12 is the oracle.
"""

import pytest

# Core API
assert hasattr(pytest, "mark"), "mark"
assert hasattr(pytest, "fixture"), "fixture"
assert hasattr(pytest, "raises"), "raises"
assert hasattr(pytest, "approx"), "approx"
assert hasattr(pytest, "warns"), "warns"
assert hasattr(pytest, "skip"), "skip"
assert hasattr(pytest, "fail"), "fail"
assert hasattr(pytest, "importorskip"), "importorskip"
assert hasattr(pytest, "param"), "param"
assert hasattr(pytest, "main"), "main"
assert hasattr(pytest, "exit"), "exit"

# pytest.mark has standard markers
assert hasattr(pytest.mark, "parametrize"), "mark.parametrize"
assert hasattr(pytest.mark, "skip"), "mark.skip"
assert hasattr(pytest.mark, "xfail"), "mark.xfail"
assert hasattr(pytest.mark, "skipif"), "mark.skipif"
assert hasattr(pytest.mark, "usefixtures"), "mark.usefixtures"

# pytest.raises as context manager
with pytest.raises(ValueError):
    raise ValueError("test")

with pytest.raises(ValueError, match="test"):
    raise ValueError("test error")

# match attribute on ExceptionInfo
with pytest.raises(ZeroDivisionError) as _exc_info:
    1 / 0
assert _exc_info.type is ZeroDivisionError, "exc_info.type"
assert _exc_info.value is not None, "exc_info.value"

# pytest.approx
assert 0.1 + 0.2 == pytest.approx(0.3), "approx 0.3"
assert [0.1, 0.2] == pytest.approx([0.1, 0.2]), "approx list"
assert 1.0 == pytest.approx(1.0, rel=1e-6), "approx with rel"

# pytest.param
_p = pytest.param(1, 2, id="case1")
assert hasattr(_p, "id"), "param.id"
assert _p.id == "case1", f"param id = {_p.id!r}"

# Module attributes stable
_raises_ref = pytest.raises
assert pytest.raises is _raises_ref, "raises stable"
_approx_ref = pytest.approx
assert pytest.approx is _approx_ref, "approx stable"
_mark_ref = pytest.mark
assert pytest.mark is _mark_ref, "mark stable"

print("surface OK")
