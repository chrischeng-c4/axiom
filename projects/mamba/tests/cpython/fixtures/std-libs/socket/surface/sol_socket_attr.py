# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "sol_socket_attr"
# subject = "socket"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket: sol_socket_attr (surface)."""
import socket

assert hasattr(socket, "SOL_SOCKET")
print("sol_socket_attr OK")
