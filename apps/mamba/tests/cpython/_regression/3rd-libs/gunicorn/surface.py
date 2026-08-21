"""Surface contract for third-party gunicorn package.

# type-regime: monomorphic

Probes: gunicorn.__version__, gunicorn.SERVER, gunicorn.SERVER_SOFTWARE,
gunicorn.version_info, gunicorn.app.base.
CPython 3.12 is the oracle.
"""

import gunicorn  # type: ignore[import]

# Core API
assert hasattr(gunicorn, "__version__"), "__version__"
assert hasattr(gunicorn, "SERVER"), "SERVER"
assert hasattr(gunicorn, "SERVER_SOFTWARE"), "SERVER_SOFTWARE"
assert hasattr(gunicorn, "version_info"), "version_info"

# Version
assert isinstance(gunicorn.__version__, str), \
    f"version type = {type(gunicorn.__version__)!r}"

# SERVER is a str constant
assert isinstance(gunicorn.SERVER, str), \
    f"SERVER type = {type(gunicorn.SERVER)!r}"
assert "gunicorn" in gunicorn.SERVER.lower(), \
    f"SERVER value = {gunicorn.SERVER!r}"

# SERVER_SOFTWARE includes version
assert isinstance(gunicorn.SERVER_SOFTWARE, str), \
    f"SERVER_SOFTWARE type = {type(gunicorn.SERVER_SOFTWARE)!r}"

# version_info is a tuple
assert isinstance(gunicorn.version_info, tuple), \
    f"version_info type = {type(gunicorn.version_info)!r}"
assert len(gunicorn.version_info) >= 2, \
    f"version_info len = {len(gunicorn.version_info)}"

# app.base module
import gunicorn.app.base as _base  # type: ignore[import]
assert hasattr(_base, "BaseApplication"), "BaseApplication"
assert callable(_base.BaseApplication), "BaseApplication callable"

# Module attributes stable
_v_ref = gunicorn.__version__
assert gunicorn.__version__ is _v_ref, "__version__ stable"
_s_ref = gunicorn.SERVER
assert gunicorn.SERVER is _s_ref, "SERVER stable"
_sw_ref = gunicorn.SERVER_SOFTWARE
assert gunicorn.SERVER_SOFTWARE is _sw_ref, "SERVER_SOFTWARE stable"
_vi_ref = gunicorn.version_info
assert gunicorn.version_info is _vi_ref, "version_info stable"

print("surface OK")
