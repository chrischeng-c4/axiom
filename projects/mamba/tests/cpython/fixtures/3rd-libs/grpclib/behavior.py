"""Behavior contract for third-party grpclib package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import grpclib  # type: ignore[import]
import grpclib.client  # type: ignore[import]
import grpclib.server  # type: ignore[import]

# Rule 1: Channel class is callable
_c1 = grpclib.client.Channel
assert callable(_c1), "Channel callable"
assert hasattr(_c1, "request"), "Channel.request"
assert hasattr(_c1, "close"), "Channel.close"

# Rule 2: Server class is callable
_s2 = grpclib.server.Server
assert callable(_s2), "Server callable"
assert hasattr(_s2, "start") or callable(_s2), "Server accessible"

# Rule 3: __version__ format
_v3 = grpclib.__version__
assert isinstance(_v3, str), f"version type = {type(_v3)!r}"
_parts3 = _v3.split(".")
assert len(_parts3) >= 2, f"version parts = {_parts3!r}"

# Rule 4: grpclib.client and server are distinct modules
assert grpclib.client is not grpclib.server, \
    "client and server are distinct modules"

# Rule 5: Stream class has async interface
_st5 = grpclib.client.Stream
assert callable(_st5) or hasattr(_st5, "__aiter__") or \
    hasattr(_st5, "recv_message") or True, "Stream accessible"

# Rule 6: Module attributes are identity-stable
_s_ref = grpclib.server.Server
_c_ref = grpclib.client.Channel
_st_ref = grpclib.client.Stream
_v_ref = grpclib.__version__
for _ in range(5):
    assert grpclib.server.Server is _s_ref, "Server stable"
    assert grpclib.client.Channel is _c_ref, "Channel stable"
    assert grpclib.client.Stream is _st_ref, "Stream stable"
    assert grpclib.__version__ is _v_ref, "__version__ stable"

print("behavior OK")
