# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "gethostname_is_callable"
# subject = "socket.gethostname"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.gethostname: gethostname_is_callable (surface)."""
import socket

assert callable(socket.gethostname)
print("gethostname_is_callable OK")
