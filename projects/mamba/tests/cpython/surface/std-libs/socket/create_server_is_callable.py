# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "create_server_is_callable"
# subject = "socket.create_server"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.create_server: create_server_is_callable (surface)."""
import socket

assert callable(socket.create_server)
print("create_server_is_callable OK")
