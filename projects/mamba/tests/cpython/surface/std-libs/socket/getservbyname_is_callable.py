# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "getservbyname_is_callable"
# subject = "socket.getservbyname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.getservbyname: getservbyname_is_callable (surface)."""
import socket

assert callable(socket.getservbyname)
print("getservbyname_is_callable OK")
