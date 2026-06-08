"""Behavior contract for third-party botocore package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import botocore  # type: ignore[import]
import botocore.session  # type: ignore[import]
import botocore.exceptions  # type: ignore[import]

# Rule 1: get_session() returns a Session object
_sess1 = botocore.session.get_session()
assert _sess1 is not None, "get_session returns session"
assert hasattr(_sess1, "get_available_services"), "has get_available_services"

# Rule 2: Session.get_available_services returns list with known services
_svcs2 = _sess1.get_available_services()
assert isinstance(_svcs2, list), f"services type = {type(_svcs2)!r}"
assert "s3" in _svcs2, "s3 available"
assert "ec2" in _svcs2, "ec2 available"

# Rule 3: BotoCoreError is the base exception
_e3 = botocore.exceptions.BotoCoreError()
assert isinstance(_e3, Exception), "BotoCoreError < Exception"

# Rule 4: ClientError requires error_response and operation_name
_e4 = botocore.exceptions.ClientError(
    {"Error": {"Code": "NoSuchBucket", "Message": "bucket not found"}},
    "GetObject"
)
assert hasattr(_e4, "response"), "ClientError.response"
assert _e4.response["Error"]["Code"] == "NoSuchBucket", \
    f"error code = {_e4.response['Error']['Code']!r}"
assert _e4.operation_name == "GetObject", \
    f"operation = {_e4.operation_name!r}"

# Rule 5: NoCredentialsError is an exception
_raised5 = False
try:
    raise botocore.exceptions.NoCredentialsError()
except botocore.exceptions.BotoCoreError:
    _raised5 = True
assert _raised5, "NoCredentialsError is BotoCoreError"

# Rule 6: Module attributes are identity-stable
_s_ref = botocore.session
_c_ref = botocore.client
_e_ref = botocore.exceptions
_ef_ref = botocore.errorfactory
for _ in range(5):
    assert botocore.session is _s_ref, "session stable"
    assert botocore.client is _c_ref, "client stable"
    assert botocore.exceptions is _e_ref, "exceptions stable"
    assert botocore.errorfactory is _ef_ref, "errorfactory stable"

print("behavior OK")
