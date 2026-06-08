"""Surface contract for third-party boto3 package.

# type-regime: monomorphic

Probes: boto3.client, boto3.resource, boto3.Session,
boto3.setup_default_session, boto3.__version__.
CPython 3.12 is the oracle.
"""

import boto3  # type: ignore[import]

# Core API
assert hasattr(boto3, "client"), "client"
assert hasattr(boto3, "resource"), "resource"
assert hasattr(boto3, "Session"), "Session"
assert hasattr(boto3, "setup_default_session"), "setup_default_session"
assert hasattr(boto3, "__version__"), "__version__"
assert hasattr(boto3, "NullHandler"), "NullHandler"

# Version
assert isinstance(boto3.__version__, str), \
    f"version type = {type(boto3.__version__)!r}"

# Callables
assert callable(boto3.client), "client callable"
assert callable(boto3.resource), "resource callable"
assert callable(boto3.Session), "Session callable"
assert callable(boto3.setup_default_session), "setup_default_session callable"

# Session construction
_sess = boto3.Session(region_name="us-east-1")
assert hasattr(_sess, "client"), "session.client"
assert hasattr(_sess, "resource"), "session.resource"
assert hasattr(_sess, "get_available_services"), "session.get_available_services"
assert hasattr(_sess, "region_name"), "session.region_name"
assert _sess.region_name == "us-east-1", f"region = {_sess.region_name!r}"

# Session.get_available_services returns list of strings
_svcs = _sess.get_available_services()
assert isinstance(_svcs, list), f"services type = {type(_svcs)!r}"
assert len(_svcs) > 0, "services not empty"
assert "s3" in _svcs, f"s3 in services: {_svcs[:5]!r}..."

# Module attributes stable
_c_ref = boto3.client
assert boto3.client is _c_ref, "client stable"
_r_ref = boto3.resource
assert boto3.resource is _r_ref, "resource stable"
_s_ref = boto3.Session
assert boto3.Session is _s_ref, "Session stable"
_sd_ref = boto3.setup_default_session
assert boto3.setup_default_session is _sd_ref, "setup_default_session stable"

print("surface OK")
