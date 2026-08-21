# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "recv_on_closed_socket_raises"
# subject = "socket.socket"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: recv_on_closed_socket_raises (errors)."""
import socket

_raised = False
try:
    (lambda s: (s.close(), s.recv(1024)))(socket.socket())
except OSError:
    _raised = True
assert _raised, "recv_on_closed_socket_raises: expected OSError"
print("recv_on_closed_socket_raises OK")
