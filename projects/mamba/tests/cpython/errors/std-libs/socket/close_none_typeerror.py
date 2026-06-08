# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "close_none_typeerror"
# subject = "socket.close"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.close: close_none_typeerror (errors)."""
import socket

_raised = False
try:
    socket.close(None)
except TypeError:
    _raised = True
assert _raised, "close_none_typeerror: expected TypeError"
print("close_none_typeerror OK")
