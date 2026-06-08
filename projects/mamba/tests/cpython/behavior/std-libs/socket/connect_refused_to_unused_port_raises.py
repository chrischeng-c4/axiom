# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "connect_refused_to_unused_port_raises"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: connecting (with a short timeout) to loopback port 1 raises a ConnectionRefusedError / timeout / OSError"""
import socket

_raised = False
try:
    _s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    _s.settimeout(1.0)
    _s.connect(("127.0.0.1", 1))  # port 1 is privileged/unused
    _s.close()
except (ConnectionRefusedError, socket.timeout, OSError):
    _raised = True
assert _raised, "refused connection raises"
print("connect_refused_to_unused_port_raises OK")
