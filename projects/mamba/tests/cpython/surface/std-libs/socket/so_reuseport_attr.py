# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "so_reuseport_attr"
# subject = "socket"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket: so_reuseport_attr (surface)."""
import socket

assert hasattr(socket, "SO_REUSEPORT")
print("so_reuseport_attr OK")
