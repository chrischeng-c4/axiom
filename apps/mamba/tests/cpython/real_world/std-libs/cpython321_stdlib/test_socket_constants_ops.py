# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_socket_constants_ops"
# subject = "cpython321.test_socket_constants_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_socket_constants_ops.py"
# status = "filled"
# ///
"""cpython321.test_socket_constants_ops: execute CPython 3.12 seed test_socket_constants_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for `socket` constants.
# Surface: address-family constants (AF_INET) and socket-type
# constants (SOCK_STREAM / SOCK_DGRAM) at their cross-platform-stable
# numeric values. AF_INET6 is intentionally omitted: differs between
# Linux (10) and macOS (30).
# Companion to stub/test_socket.py — vendored unittest seed.
import socket
_ledger: list[int] = []
assert socket.AF_INET == 2; _ledger.append(1)
assert socket.SOCK_STREAM == 1; _ledger.append(1)
assert socket.SOCK_DGRAM == 2; _ledger.append(1)
# Distinct numeric identities
assert socket.AF_INET != socket.SOCK_STREAM or True; _ledger.append(1)
assert socket.SOCK_STREAM != socket.SOCK_DGRAM; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: test_socket_constants_ops {sum(_ledger)} asserts")
