"""Surface contract for third-party grpclib package.

# type-regime: monomorphic

Probes: grpclib.__version__, grpclib.client.Channel,
grpclib.server.Server, grpclib.client.Stream.
CPython 3.12 is the oracle.
"""

import grpclib  # type: ignore[import]
import grpclib.client  # type: ignore[import]
import grpclib.server  # type: ignore[import]

# Core API
assert hasattr(grpclib, "__version__"), "__version__"

# Version
assert isinstance(grpclib.__version__, str), \
    f"version type = {type(grpclib.__version__)!r}"

# Classes are callable
assert callable(grpclib.server.Server), "Server callable"
assert callable(grpclib.client.Channel), "Channel callable"

# client module has Channel
assert hasattr(grpclib.client, "Channel"), "client.Channel"
assert callable(grpclib.client.Channel), "client.Channel callable"

# server module has Server
assert hasattr(grpclib.server, "Server"), "server.Server"
assert callable(grpclib.server.Server), "server.Server callable"

# Stream has expected attributes
assert hasattr(grpclib.client, "Stream"), "client.Stream"
assert hasattr(grpclib.server, "Stream"), "server.Stream"

# Module attributes stable
_s_ref = grpclib.server.Server
assert grpclib.server.Server is _s_ref, "Server stable"
_c_ref = grpclib.client.Channel
assert grpclib.client.Channel is _c_ref, "Channel stable"
_st_ref = grpclib.client.Stream
assert grpclib.client.Stream is _st_ref, "Stream stable"
_v_ref = grpclib.__version__
assert grpclib.__version__ is _v_ref, "__version__ stable"

print("surface OK")
