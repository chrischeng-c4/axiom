"""Behavior contract for third-party grpcio package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import grpc  # type: ignore[import]

# Rule 1: StatusCode has correct integer values
_sc1 = grpc.StatusCode
assert _sc1.OK.value[0] == 0, f"OK = {_sc1.OK.value!r}"
assert _sc1.CANCELLED.value[0] == 1, f"CANCELLED = {_sc1.CANCELLED.value!r}"
assert _sc1.NOT_FOUND.value[0] == 5, f"NOT_FOUND = {_sc1.NOT_FOUND.value!r}"
assert _sc1.INTERNAL.value[0] == 13, f"INTERNAL = {_sc1.INTERNAL.value!r}"
assert _sc1.UNAUTHENTICATED.value[0] == 16, \
    f"UNAUTHENTICATED = {_sc1.UNAUTHENTICATED.value!r}"

# Rule 2: RpcError is the base exception for gRPC errors
assert issubclass(grpc.RpcError, Exception), "RpcError < Exception"

# Rule 3: insecure_channel is callable
assert callable(grpc.insecure_channel), "insecure_channel callable"
assert callable(grpc.secure_channel), "secure_channel callable"

# Rule 4: ssl_channel_credentials is callable
assert callable(grpc.ssl_channel_credentials), "ssl_channel_credentials callable"
_creds4 = grpc.ssl_channel_credentials()
assert _creds4 is not None, "ssl_channel_credentials returns non-None"

# Rule 5: server() creates server object
import concurrent.futures
_srv5 = grpc.server(concurrent.futures.ThreadPoolExecutor(max_workers=10))
assert hasattr(_srv5, "add_insecure_port"), "server.add_insecure_port"
assert hasattr(_srv5, "start"), "server.start"
assert hasattr(_srv5, "stop"), "server.stop"
_srv5.stop(0)

# Rule 6: Module attributes are identity-stable
_s_ref = grpc.Server
_c_ref = grpc.Channel
_ic_ref = grpc.insecure_channel
_v_ref = grpc.__version__
for _ in range(5):
    assert grpc.Server is _s_ref, "Server stable"
    assert grpc.Channel is _c_ref, "Channel stable"
    assert grpc.insecure_channel is _ic_ref, "insecure_channel stable"
    assert grpc.__version__ is _v_ref, "__version__ stable"

print("behavior OK")
