# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "nonblocking_accept_raises_blockingio"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: a listening socket put in setblocking(False) with no pending connection raises BlockingIOError (a socket.error) from accept()"""
import socket

_nb = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
_nb.bind(("127.0.0.1", 0))
_nb.listen(1)
_nb.setblocking(False)
_raised = False
try:
    _nb.accept()
except (BlockingIOError, socket.error):
    _raised = True
_nb.close()
assert _raised, "non-blocking accept raises BlockingIOError"
print("nonblocking_accept_raises_blockingio OK")
