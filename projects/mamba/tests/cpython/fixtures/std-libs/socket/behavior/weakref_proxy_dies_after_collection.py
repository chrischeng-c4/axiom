# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "behavior"
# case = "weakref_proxy_dies_after_collection"
# subject = "socket.socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket.socket: a weakref.proxy mirrors a live socket's fileno but raises ReferenceError once the socket is dropped and gc-collected"""
import gc
import socket
from weakref import proxy

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
    p = proxy(s)
    assert p.fileno() == s.fileno(), "proxy should mirror the live socket"
s = None
gc.collect()
dead = False
try:
    p.fileno()
except ReferenceError:
    dead = True
assert dead, "proxy should raise ReferenceError after collection"
print("weakref_proxy_dies_after_collection OK")
