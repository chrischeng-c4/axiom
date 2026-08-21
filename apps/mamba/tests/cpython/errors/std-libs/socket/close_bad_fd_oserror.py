# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "close_bad_fd_oserror"
# subject = "socket.close"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.close: close_bad_fd_oserror (errors)."""
import socket

_raised = False
try:
    socket.close(-1)
except OSError:
    _raised = True
assert _raised, "close_bad_fd_oserror: expected OSError"
print("close_bad_fd_oserror OK")
