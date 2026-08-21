# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "errors"
# case = "timeout_is_timeouterror_alias"
# subject = "socket.timeout"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.timeout: socket.timeout is the builtin TimeoutError alias (Python 3.10+)"""
import socket

assert socket.timeout is TimeoutError, f"socket.timeout = {socket.timeout!r}"
print("timeout_is_timeouterror_alias OK")
