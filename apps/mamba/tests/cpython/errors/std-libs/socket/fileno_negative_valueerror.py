# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "fileno_negative_valueerror"
# subject = "socket.socket"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: fileno_negative_valueerror (errors)."""
import socket

_raised = False
try:
    socket.socket(socket.AF_INET, socket.SOCK_STREAM, fileno=-1)
except ValueError:
    _raised = True
assert _raised, "fileno_negative_valueerror: expected ValueError"
print("fileno_negative_valueerror OK")
