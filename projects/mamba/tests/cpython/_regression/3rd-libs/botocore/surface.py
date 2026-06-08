"""Surface contract for third-party botocore package.

# type-regime: monomorphic

Probes: botocore.session, botocore.client, botocore.exceptions,
botocore.errorfactory, botocore.__version__.
CPython 3.12 is the oracle.
"""

import botocore  # type: ignore[import]
import botocore.session  # type: ignore[import]
import botocore.exceptions  # type: ignore[import]

# Core API
assert hasattr(botocore, "session"), "session"
assert hasattr(botocore, "client"), "client"
assert hasattr(botocore, "exceptions"), "exceptions"
assert hasattr(botocore, "errorfactory"), "errorfactory"
assert hasattr(botocore, "__version__"), "__version__"

# Version
assert isinstance(botocore.__version__, str), \
    f"version type = {type(botocore.__version__)!r}"

# session module has get_session
assert hasattr(botocore.session, "get_session"), "get_session"
assert callable(botocore.session.get_session), "get_session callable"

# exceptions module has BotoCoreError
assert hasattr(botocore.exceptions, "BotoCoreError"), "BotoCoreError"
assert hasattr(botocore.exceptions, "ClientError"), "ClientError"
assert hasattr(botocore.exceptions, "NoCredentialsError"), "NoCredentialsError"
assert hasattr(botocore.exceptions, "EndpointConnectionError"), \
    "EndpointConnectionError"
assert issubclass(botocore.exceptions.BotoCoreError, Exception), \
    "BotoCoreError < Exception"
assert issubclass(botocore.exceptions.ClientError, Exception), \
    "ClientError < Exception"

# Session from get_session
_sess = botocore.session.get_session()
assert hasattr(_sess, "get_service_model"), "session.get_service_model"
assert hasattr(_sess, "get_available_services"), "session.get_available_services"

# Module attributes stable
_s_ref = botocore.session
assert botocore.session is _s_ref, "session stable"
_c_ref = botocore.client
assert botocore.client is _c_ref, "client stable"
_e_ref = botocore.exceptions
assert botocore.exceptions is _e_ref, "exceptions stable"
_ef_ref = botocore.errorfactory
assert botocore.errorfactory is _ef_ref, "errorfactory stable"

print("surface OK")
