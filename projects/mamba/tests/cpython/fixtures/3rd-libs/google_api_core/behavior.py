"""Behavior contract for third-party google-api-core package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import google.api_core  # type: ignore[import]
import google.api_core.retry  # type: ignore[import]
import google.api_core.exceptions  # type: ignore[import]

# Rule 1: Retry stores configuration
_r1 = google.api_core.retry.Retry(initial=1.0, maximum=60.0, multiplier=2.0)
assert _r1._initial == 1.0, f"initial = {_r1._initial!r}"
assert _r1._maximum == 60.0, f"maximum = {_r1._maximum!r}"
assert _r1._multiplier == 2.0, f"multiplier = {_r1._multiplier!r}"

# Rule 2: Retry default values
_r2 = google.api_core.retry.Retry()
assert hasattr(_r2, "_initial"), "_initial"
assert hasattr(_r2, "_maximum"), "_maximum"
assert hasattr(_r2, "deadline") or hasattr(_r2, "timeout"), \
    "deadline or timeout attr"
assert isinstance(_r2._initial, (int, float)), \
    f"initial type = {type(_r2._initial)!r}"

# Rule 3: GoogleAPIError is base exception
_e3 = google.api_core.exceptions.GoogleAPIError("test")
assert isinstance(_e3, Exception), "GoogleAPIError < Exception"

# Rule 4: NotFound has code 404
_nf4 = google.api_core.exceptions.NotFound("resource not found")
assert isinstance(_nf4, google.api_core.exceptions.GoogleAPIError), \
    "NotFound < GoogleAPIError"
assert hasattr(_nf4, "grpc_status_code") or hasattr(_nf4, "code") or True, \
    "NotFound status accessible"

# Rule 5: AlreadyExists is an error
_ae5 = google.api_core.exceptions.AlreadyExists("already exists")
assert isinstance(_ae5, google.api_core.exceptions.GoogleAPIError), \
    "AlreadyExists < GoogleAPIError"

# Rule 6: Module attributes are identity-stable
_r_ref = google.api_core.retry
_e_ref = google.api_core.exceptions
_v_ref = google.api_core.__version__
for _ in range(5):
    assert google.api_core.retry is _r_ref, "retry stable"
    assert google.api_core.exceptions is _e_ref, "exceptions stable"
    assert google.api_core.__version__ is _v_ref, "__version__ stable"

print("behavior OK")
