# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "inheritable_flag_toggles_and_dup"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: sockets are non-inheritable by default, set_inheritable toggles the flag both ways, and dup() yields an independent non-inheritable live descriptor"""
import socket

# Sockets are non-inheritable by default (FD_CLOEXEC set).
sock = socket.socket()
assert sock.get_inheritable() is False, "default should be non-inheritable"

# set_inheritable toggles the flag both ways.
sock.set_inheritable(True)
assert sock.get_inheritable() is True, "after set_inheritable(True)"
sock.set_inheritable(False)
assert sock.get_inheritable() is False, "after set_inheritable(False)"

# A duplicated socket is its own non-inheritable descriptor and survives
# the original being closed.
dup = sock.dup()
sock.close()
assert dup.get_inheritable() is False, "dup() result should be non-inheritable"
assert dup.fileno() >= 0, "dup() should have a live descriptor"
dup.close()
print("inheritable_flag_toggles_and_dup OK")
