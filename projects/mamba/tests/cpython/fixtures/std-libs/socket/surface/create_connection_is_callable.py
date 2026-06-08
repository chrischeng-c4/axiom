# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "create_connection_is_callable"
# subject = "socket.create_connection"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.create_connection: create_connection_is_callable (surface)."""
import socket

assert callable(socket.create_connection)
print("create_connection_is_callable OK")
