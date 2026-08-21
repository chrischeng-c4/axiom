# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "listen_backlog_and_type_flags_stable"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: listen() accepts a zero/negative/absent backlog, and the reported .type stays SOCK_STREAM across SOCK_NONBLOCK/SOCK_CLOEXEC request flags and timeout/blocking changes"""
import socket

# listen() accepts a zero or negative backlog and a no-argument call.
for backlog in (0, -1):
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as srv:
        srv.bind(("127.0.0.1", 0))
        srv.listen(backlog)
with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as srv:
    srv.bind(("127.0.0.1", 0))
    srv.listen()

# The reported .type stays SOCK_STREAM even when extra OS flags are requested
# and across timeout/blocking changes (the flags do not leak into .type).
nonblock = getattr(socket, "SOCK_NONBLOCK", 0)
cloexec = getattr(socket, "SOCK_CLOEXEC", 0)
with socket.socket(socket.AF_INET, socket.SOCK_STREAM | nonblock | cloexec) as s:
    assert s.type == socket.SOCK_STREAM, f"initial type = {s.type!r}"
    for change in (lambda: s.settimeout(1), lambda: s.settimeout(0),
                   lambda: s.setblocking(True), lambda: s.setblocking(False)):
        change()
        assert s.type == socket.SOCK_STREAM, f"type drifted: {s.type!r}"
print("listen_backlog_and_type_flags_stable OK")
