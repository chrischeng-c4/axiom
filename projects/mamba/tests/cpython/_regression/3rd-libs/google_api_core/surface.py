"""Surface contract for third-party google-api-core package.

# type-regime: monomorphic

Probes: google.api_core.retry, google.api_core.timeout,
google.api_core.exceptions, google.api_core.__version__.
CPython 3.12 is the oracle.
"""

import google.api_core  # type: ignore[import]
import google.api_core.retry  # type: ignore[import]
import google.api_core.exceptions  # type: ignore[import]

# Core API
assert hasattr(google.api_core, "retry"), "retry"
assert hasattr(google.api_core, "exceptions"), "exceptions"
assert hasattr(google.api_core, "__version__"), "__version__"

# Version
assert isinstance(google.api_core.__version__, str), \
    f"version type = {type(google.api_core.__version__)!r}"

# retry module
assert hasattr(google.api_core.retry, "Retry"), "retry.Retry"
assert callable(google.api_core.retry.Retry), "Retry callable"

# Retry construction
_retry = google.api_core.retry.Retry(
    initial=1.0,
    maximum=60.0,
    multiplier=2.0,
)
assert hasattr(_retry, "_initial"), "retry._initial"
assert hasattr(_retry, "_maximum"), "retry._maximum"
assert hasattr(_retry, "_multiplier"), "retry._multiplier"

# exceptions module
assert hasattr(google.api_core.exceptions, "GoogleAPIError"), "GoogleAPIError"
assert hasattr(google.api_core.exceptions, "NotFound"), "NotFound"
assert hasattr(google.api_core.exceptions, "AlreadyExists"), "AlreadyExists"
assert hasattr(google.api_core.exceptions, "ResourceExhausted"), \
    "ResourceExhausted"
assert issubclass(google.api_core.exceptions.GoogleAPIError, Exception), \
    "GoogleAPIError < Exception"
assert issubclass(google.api_core.exceptions.NotFound,
                  google.api_core.exceptions.GoogleAPIError), \
    "NotFound < GoogleAPIError"

# Module attributes stable
_r_ref = google.api_core.retry
assert google.api_core.retry is _r_ref, "retry stable"
_e_ref = google.api_core.exceptions
assert google.api_core.exceptions is _e_ref, "exceptions stable"
_v_ref = google.api_core.__version__
assert google.api_core.__version__ is _v_ref, "__version__ stable"

print("surface OK")
