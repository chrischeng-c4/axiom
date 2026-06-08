"""Surface contract for third-party grpcio package.

# type-regime: monomorphic

Probes: grpc.Server, grpc.Channel, grpc.insecure_channel,
grpc.__version__, grpc.StatusCode, grpc.RpcError.
CPython 3.12 is the oracle.
"""

import grpc  # type: ignore[import]

# Core API
assert hasattr(grpc, "Server"), "Server"
assert hasattr(grpc, "Channel"), "Channel"
assert hasattr(grpc, "insecure_channel"), "insecure_channel"
assert hasattr(grpc, "__version__"), "__version__"
assert hasattr(grpc, "StatusCode"), "StatusCode"
assert hasattr(grpc, "RpcError"), "RpcError"
assert hasattr(grpc, "server"), "server"
assert hasattr(grpc, "secure_channel"), "secure_channel"
assert hasattr(grpc, "ssl_channel_credentials"), "ssl_channel_credentials"

# Version
assert isinstance(grpc.__version__, str), \
    f"version type = {type(grpc.__version__)!r}"

# Classes / callables
assert callable(grpc.insecure_channel), "insecure_channel callable"
assert callable(grpc.server), "server callable"
assert callable(grpc.secure_channel), "secure_channel callable"

# StatusCode has expected members
assert hasattr(grpc.StatusCode, "OK"), "StatusCode.OK"
assert hasattr(grpc.StatusCode, "NOT_FOUND"), "StatusCode.NOT_FOUND"
assert hasattr(grpc.StatusCode, "INTERNAL"), "StatusCode.INTERNAL"
assert hasattr(grpc.StatusCode, "UNAUTHENTICATED"), "StatusCode.UNAUTHENTICATED"

# StatusCode.OK value
assert grpc.StatusCode.OK.value[0] == 0, \
    f"OK.value = {grpc.StatusCode.OK.value!r}"

# RpcError hierarchy
assert issubclass(grpc.RpcError, Exception), "RpcError < Exception"

# Module attributes stable
_s_ref = grpc.Server
assert grpc.Server is _s_ref, "Server stable"
_c_ref = grpc.Channel
assert grpc.Channel is _c_ref, "Channel stable"
_ic_ref = grpc.insecure_channel
assert grpc.insecure_channel is _ic_ref, "insecure_channel stable"
_v_ref = grpc.__version__
assert grpc.__version__ is _v_ref, "__version__ stable"

print("surface OK")
