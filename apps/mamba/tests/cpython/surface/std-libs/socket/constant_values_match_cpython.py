# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "socket"
# dimension = "surface"
# case = "constant_values_match_cpython"
# subject = "socket"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""socket: the well-known integer constants pin to their CPython values: AF_INET == 2, SOCK_STREAM == 1, SOCK_DGRAM == 2"""
import socket

assert socket.AF_INET == 2, f"AF_INET = {socket.AF_INET!r}"
assert socket.SOCK_STREAM == 1, f"SOCK_STREAM = {socket.SOCK_STREAM!r}"
assert socket.SOCK_DGRAM == 2, f"SOCK_DGRAM = {socket.SOCK_DGRAM!r}"
print("constant_values_match_cpython OK")
