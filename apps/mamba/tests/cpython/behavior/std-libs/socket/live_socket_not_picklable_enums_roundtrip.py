# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "live_socket_not_picklable_enums_roundtrip"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: a live socket object is unpicklable (TypeError) at every protocol, while the AddressFamily/SocketKind IntEnum constants round-trip through pickle preserving value"""
import pickle
import socket

# A live socket object cannot be pickled (TypeError at every protocol).
sock = socket.socket()
with sock:
    for protocol in range(pickle.HIGHEST_PROTOCOL + 1):
        raised = False
        try:
            pickle.dumps(sock, protocol)
        except TypeError:
            raised = True
        assert raised, f"socket pickling should fail at protocol {protocol}"

# The IntEnum constants round-trip through pickle, preserving value/identity.
for protocol in range(pickle.HIGHEST_PROTOCOL + 1):
    family = pickle.loads(pickle.dumps(socket.AF_INET, protocol))
    assert family == socket.AF_INET, f"AF_INET round-trip at {protocol}: {family!r}"
    kind = pickle.loads(pickle.dumps(socket.SOCK_STREAM, protocol))
    assert kind == socket.SOCK_STREAM, f"SOCK_STREAM round-trip at {protocol}: {kind!r}"
print("live_socket_not_picklable_enums_roundtrip OK")
