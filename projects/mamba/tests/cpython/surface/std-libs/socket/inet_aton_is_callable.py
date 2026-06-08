# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "inet_aton_is_callable"
# subject = "socket.inet_aton"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.inet_aton: inet_aton_is_callable (surface)."""
import socket

assert callable(socket.inet_aton)
print("inet_aton_is_callable OK")
