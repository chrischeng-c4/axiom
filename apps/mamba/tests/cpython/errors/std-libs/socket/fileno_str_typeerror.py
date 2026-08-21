# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "fileno_str_typeerror"
# subject = "socket.socket"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: fileno_str_typeerror (errors)."""
import socket

_raised = False
try:
    socket.socket(socket.AF_INET, socket.SOCK_STREAM, fileno="foo")
except TypeError:
    _raised = True
assert _raised, "fileno_str_typeerror: expected TypeError"
print("fileno_str_typeerror OK")
