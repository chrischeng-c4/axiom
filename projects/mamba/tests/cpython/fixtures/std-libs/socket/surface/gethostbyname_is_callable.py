# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "gethostbyname_is_callable"
# subject = "socket.gethostbyname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.gethostbyname: gethostbyname_is_callable (surface)."""
import socket

assert callable(socket.gethostbyname)
print("gethostbyname_is_callable OK")
