# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "shut_rdwr_attr"
# subject = "socket"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket: shut_rdwr_attr (surface)."""
import socket

assert hasattr(socket, "SHUT_RDWR")
print("shut_rdwr_attr OK")
