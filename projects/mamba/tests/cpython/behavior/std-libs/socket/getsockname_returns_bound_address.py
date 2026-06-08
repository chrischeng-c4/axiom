# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "getsockname_returns_bound_address"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: after bind(('127.0.0.1', 0)) getsockname() reports host '127.0.0.1' and a positive integer ephemeral port"""
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
_s.bind(("127.0.0.1", 0))
_name = _s.getsockname()
assert _name[0] == "127.0.0.1", f"getsockname host = {_name[0]!r}"
assert isinstance(_name[1], int), f"getsockname port = {_name[1]!r}"
assert _name[1] > 0, "port is positive"
_s.close()
print("getsockname_returns_bound_address OK")
