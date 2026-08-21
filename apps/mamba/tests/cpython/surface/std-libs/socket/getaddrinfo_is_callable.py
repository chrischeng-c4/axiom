# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "getaddrinfo_is_callable"
# subject = "socket.getaddrinfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.getaddrinfo: getaddrinfo_is_callable (surface)."""
import socket

assert callable(socket.getaddrinfo)
print("getaddrinfo_is_callable OK")
