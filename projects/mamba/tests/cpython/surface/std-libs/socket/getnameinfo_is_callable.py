# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "getnameinfo_is_callable"
# subject = "socket.getnameinfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.getnameinfo: getnameinfo_is_callable (surface)."""
import socket

assert callable(socket.getnameinfo)
print("getnameinfo_is_callable OK")
