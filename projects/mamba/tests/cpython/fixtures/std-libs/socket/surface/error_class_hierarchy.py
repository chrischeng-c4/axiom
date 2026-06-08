# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "error_class_hierarchy"
# subject = "socket.error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.error: socket.error is OSError, and socket.timeout / socket.gaierror / socket.herror are all subclasses of socket.error"""
import socket

assert socket.error is OSError, "socket.error is OSError"
assert issubclass(socket.timeout, socket.error), "timeout < socket.error"
assert issubclass(socket.gaierror, socket.error), "gaierror < socket.error"
assert issubclass(socket.herror, socket.error), "herror < socket.error"
print("error_class_hierarchy OK")
