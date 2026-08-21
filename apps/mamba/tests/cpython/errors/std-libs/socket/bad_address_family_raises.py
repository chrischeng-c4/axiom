# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "bad_address_family_raises"
# subject = "socket.socket"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: bad_address_family_raises (errors)."""
import socket

_raised = False
try:
    socket.socket(99999, socket.SOCK_STREAM)
except OSError:
    _raised = True
assert _raised, "bad_address_family_raises: expected OSError"
print("bad_address_family_raises OK")
