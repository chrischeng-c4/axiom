# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "repr_reports_fd_family_type_and_state"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: repr() of a socket shows fd/family/type/proto, gains laddr after bind, and shows [closed] after close; the family/type IntEnum members carry rich repr and numeric str"""
import socket

# An open, unbound socket reports fd/family/type/proto and no remote address.
s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
r = repr(s)
assert ("fd=%i" % s.fileno()) in r, f"fd missing: {r}"
assert ("family=%s" % socket.AF_INET) in r, f"family missing: {r}"
assert ("type=%s" % socket.SOCK_STREAM) in r, f"type missing: {r}"
assert "proto=0" in r, f"proto missing: {r}"
assert "raddr" not in r, f"unexpected raddr: {r}"

# After binding, the local address shows up in the repr.
s.bind(("127.0.0.1", 0))
r = repr(s)
assert "laddr" in r, f"laddr missing after bind: {r}"
assert str(s.getsockname()) in r, f"sockname missing: {r}"

# A closed socket reports [closed] and drops the local address.
s.close()
r = repr(s)
assert "[closed]" in r, f"closed marker missing: {r}"
assert "laddr" not in r, f"laddr leaked after close: {r}"

# socket.family / socket.type are IntEnum members with rich repr.
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s2:
    assert repr(s2.family) == "<AddressFamily.AF_INET: %r>" % s2.family.value, repr(s2.family)
    assert repr(s2.type) == "<SocketKind.SOCK_STREAM: %r>" % s2.type.value, repr(s2.type)
    # str() of an IntEnum member is its numeric value (Python 3.11+ behavior).
    assert str(s2.family) == str(s2.family.value), str(s2.family)
    assert str(s2.type) == str(s2.type.value), str(s2.type)
print("repr_reports_fd_family_type_and_state OK")
