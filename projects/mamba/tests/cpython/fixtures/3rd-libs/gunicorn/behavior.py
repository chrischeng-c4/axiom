"""Behavior contract for third-party gunicorn package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import gunicorn  # type: ignore[import]

# Rule 1: SERVER is a string containing "gunicorn" name
_s1 = gunicorn.SERVER
assert isinstance(_s1, str), f"SERVER type = {type(_s1)!r}"
assert "gunicorn" in _s1.lower(), f"SERVER = {_s1!r}"

# Rule 2: SERVER_SOFTWARE contains version info
_sw2 = gunicorn.SERVER_SOFTWARE
assert isinstance(_sw2, str), f"SERVER_SOFTWARE type = {type(_sw2)!r}"
assert "gunicorn" in _sw2.lower() or gunicorn.__version__ in _sw2, \
    f"SERVER_SOFTWARE = {_sw2!r}"

# Rule 3: version_info is a tuple of integers
_vi3 = gunicorn.version_info
assert isinstance(_vi3, tuple), f"version_info type = {type(_vi3)!r}"
assert all(isinstance(x, int) for x in _vi3[:3]), \
    f"version_info elements = {_vi3!r}"
assert _vi3[0] >= 20, f"major version = {_vi3[0]!r}"

# Rule 4: __version__ matches version_info
_ver4 = gunicorn.__version__
_parts4 = _ver4.split(".")
assert len(_parts4) >= 2, f"version parts = {_parts4!r}"
_major4 = int(_parts4[0])
assert _major4 == gunicorn.version_info[0], \
    f"major mismatch: {_major4} vs {gunicorn.version_info[0]}"

# Rule 5: BaseApplication is a class
import gunicorn.app.base as _base  # type: ignore[import]
assert callable(_base.BaseApplication), "BaseApplication callable"
assert hasattr(_base.BaseApplication, "run"), "BaseApplication.run"
assert hasattr(_base.BaseApplication, "load_config"), "BaseApplication.load_config"

# Rule 6: Module attributes are identity-stable
_v_ref = gunicorn.__version__
_s_ref = gunicorn.SERVER
_sw_ref = gunicorn.SERVER_SOFTWARE
_vi_ref = gunicorn.version_info
for _ in range(5):
    assert gunicorn.__version__ is _v_ref, "__version__ stable"
    assert gunicorn.SERVER is _s_ref, "SERVER stable"
    assert gunicorn.SERVER_SOFTWARE is _sw_ref, "SERVER_SOFTWARE stable"
    assert gunicorn.version_info is _vi_ref, "version_info stable"

print("behavior OK")
