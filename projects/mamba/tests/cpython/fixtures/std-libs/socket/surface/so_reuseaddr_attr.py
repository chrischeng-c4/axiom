# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "so_reuseaddr_attr"
# subject = "socket"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket: so_reuseaddr_attr (surface)."""
import socket

assert hasattr(socket, "SO_REUSEADDR")
print("so_reuseaddr_attr OK")
