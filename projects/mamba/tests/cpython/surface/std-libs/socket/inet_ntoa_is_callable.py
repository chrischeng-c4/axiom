# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "inet_ntoa_is_callable"
# subject = "socket.inet_ntoa"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.inet_ntoa: inet_ntoa_is_callable (surface)."""
import socket

assert callable(socket.inet_ntoa)
print("inet_ntoa_is_callable OK")
