# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "so_reuseaddr_setsockopt_roundtrip"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: setsockopt(SOL_SOCKET, SO_REUSEADDR, 1) is observable via getsockopt as a nonzero value"""
import socket

_s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
_s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
_val = _s.getsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR)
assert _val != 0, f"SO_REUSEADDR set: {_val!r}"
_s.close()
print("so_reuseaddr_setsockopt_roundtrip OK")
