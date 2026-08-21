# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_socket_gc_pickle_value_ops"
# subject = "cpython321.test_socket_gc_pickle_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_socket_gc_pickle_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_socket_gc_pickle_value_ops: execute CPython 3.12 seed test_socket_gc_pickle_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of three
# bootstrap stdlib modules that drive every networking /
# memory-management / serialization path: `socket` (the AF_INET /
# AF_UNIX / SOCK_STREAM / SOCK_DGRAM portable address-family +
# socket-type integer sentinels), `gc` (the documented DEBUG_*
# bitflag sentinels + get_count / get_threshold tuple arity +
# collect / get_objects return-type contract), and `pickle` (the
# HIGHEST_PROTOCOL integer sentinel + dumps / loads lossless
# round-trip across the documented atomic types).
#
# The matching subset between mamba and CPython is the integer-
# constant + lossless-roundtrip layer: socket.AF_INET == 2,
# AF_UNIX == 1, SOCK_STREAM == 1, SOCK_DGRAM == 2 (POSIX integer
# sentinels common to all UNIX platforms); gc.isenabled() returns
# True; gc.get_count() / get_threshold() each return a 3-element
# tuple; gc.DEBUG_STATS == 1, DEBUG_COLLECTABLE == 2,
# DEBUG_UNCOLLECTABLE == 4, DEBUG_SAVEALL == 32, DEBUG_LEAK == 38;
# gc.collect() returns an int; gc.get_objects() returns a list;
# pickle.HIGHEST_PROTOCOL == 5; pickle.dumps / loads round-trip
# bool / None / int / tuple / list / dict / str lossless.
#
# Surface in this fixture:
#   • socket.AF_INET == 2 — POSIX IPv4 address family;
#   • socket.AF_UNIX == 1 — POSIX local-socket address family;
#   • socket.SOCK_STREAM == 1 — POSIX TCP-style socket type;
#   • socket.SOCK_DGRAM == 2 — POSIX UDP-style socket type;
#   • gc.isenabled() is True — GC active by default;
#   • len(gc.get_count()) == 3 — three-generation GC counters;
#   • len(gc.get_threshold()) == 3 — three-generation thresholds;
#   • gc.DEBUG_STATS == 1, DEBUG_COLLECTABLE == 2,
#     DEBUG_UNCOLLECTABLE == 4, DEBUG_SAVEALL == 32, DEBUG_LEAK
#     == 38 — documented debug bitflag sentinels;
#   • type(gc.collect()).__name__ == "int" — collect returns the
#     number of unreachable objects;
#   • type(gc.get_objects()).__name__ == "list";
#   • hasattr(gc, "set_threshold"), hasattr(gc, "get_threshold");
#   • pickle.HIGHEST_PROTOCOL == 5 — documented top protocol;
#   • pickle.loads(pickle.dumps(True)) is True;
#   • pickle.loads(pickle.dumps(None)) is None;
#   • pickle.loads(pickle.dumps(42)) == 42;
#   • pickle.loads(pickle.dumps((1, 2, 3))) == (1, 2, 3);
#   • pickle.loads(pickle.dumps([1, 2, 3])) == [1, 2, 3];
#   • pickle.loads(pickle.dumps({"a": 1})) == {"a": 1};
#   • pickle.loads(pickle.dumps("hello")) == "hello".
#
# Behavioral edges that DIVERGE on mamba (socket.AF_INET6 macOS
# sentinel, socket.SOCK_RAW / SOL_SOCKET / SO_REUSEADDR /
# SO_KEEPALIVE / IPPROTO_TCP / IPPROTO_UDP / SHUT_* integer
# constants, socket.socket / error / gaierror class identity,
# socket.htonl / ntohl / htons / ntohs / inet_aton / inet_ntoa
# byte-order helpers, select.POLLIN / POLLOUT / POLLERR / POLLHUP
# / POLLNVAL constants + select.select / poll helpers, weakref.
# ref / WeakValueDictionary / WeakSet / WeakKeyDictionary class
# identity + ref() round-trip, pickle.DEFAULT_PROTOCOL / dumps
# protocol-marker bytes, pickle.PickleError / PicklingError /
# UnpicklingError / Pickler / Unpickler class identity,
# gc.garbage container surface) are covered in
# `lang_socket_select_weakref_pickle_silent`.
import socket
import gc
import pickle

_ledger: list[int] = []

# 1) socket — POSIX address-family integer sentinels
assert socket.AF_INET == 2; _ledger.append(1)
assert socket.AF_UNIX == 1; _ledger.append(1)

# 2) socket — POSIX socket-type integer sentinels
assert socket.SOCK_STREAM == 1; _ledger.append(1)
assert socket.SOCK_DGRAM == 2; _ledger.append(1)

# 3) gc — runtime state predicates
assert gc.isenabled() == True; _ledger.append(1)

# 4) gc — three-generation counter / threshold arity
assert len(gc.get_count()) == 3; _ledger.append(1)
assert len(gc.get_threshold()) == 3; _ledger.append(1)

# 5) gc — documented debug bitflag sentinels
assert gc.DEBUG_STATS == 1; _ledger.append(1)
assert gc.DEBUG_COLLECTABLE == 2; _ledger.append(1)
assert gc.DEBUG_UNCOLLECTABLE == 4; _ledger.append(1)
assert gc.DEBUG_SAVEALL == 32; _ledger.append(1)
assert gc.DEBUG_LEAK == 38; _ledger.append(1)

# 6) gc — collect / get_objects return-type contract
assert type(gc.collect()).__name__ == "int"; _ledger.append(1)
assert type(gc.get_objects()).__name__ == "list"; _ledger.append(1)

# 7) pickle — HIGHEST_PROTOCOL integer sentinel
assert pickle.HIGHEST_PROTOCOL == 5; _ledger.append(1)

# 8) pickle — lossless round-trip across atomic types
assert pickle.loads(pickle.dumps(True)) == True; _ledger.append(1)
assert pickle.loads(pickle.dumps(False)) == False; _ledger.append(1)
assert pickle.loads(pickle.dumps(None)) is None; _ledger.append(1)
assert pickle.loads(pickle.dumps(42)) == 42; _ledger.append(1)
assert pickle.loads(pickle.dumps(-7)) == -7; _ledger.append(1)
assert pickle.loads(pickle.dumps(3.14)) == 3.14; _ledger.append(1)
assert pickle.loads(pickle.dumps("hello")) == "hello"; _ledger.append(1)
assert pickle.loads(pickle.dumps("")) == ""; _ledger.append(1)

# 9) pickle — lossless round-trip across container types
assert pickle.loads(pickle.dumps((1, 2, 3))) == (1, 2, 3); _ledger.append(1)
assert pickle.loads(pickle.dumps([1, 2, 3])) == [1, 2, 3]; _ledger.append(1)
assert pickle.loads(pickle.dumps([])) == []; _ledger.append(1)
assert pickle.loads(pickle.dumps({"a": 1})) == {"a": 1}; _ledger.append(1)
assert pickle.loads(pickle.dumps({})) == {}; _ledger.append(1)

# 10) hasattr surface — module-level helpers
assert hasattr(socket, "AF_INET"); _ledger.append(1)
assert hasattr(socket, "SOCK_STREAM"); _ledger.append(1)
assert hasattr(gc, "collect"); _ledger.append(1)
assert hasattr(gc, "set_threshold"); _ledger.append(1)
assert hasattr(gc, "get_threshold"); _ledger.append(1)
assert hasattr(gc, "isenabled"); _ledger.append(1)
assert hasattr(pickle, "dumps"); _ledger.append(1)
assert hasattr(pickle, "loads"); _ledger.append(1)
assert hasattr(pickle, "HIGHEST_PROTOCOL"); _ledger.append(1)

# NB: socket.AF_INET6 macOS sentinel, socket.SOCK_RAW / SOL_SOCKET
# / SO_REUSEADDR / SO_KEEPALIVE / IPPROTO_TCP / IPPROTO_UDP /
# SHUT_* integer constants, socket.socket / error / gaierror
# class identity, socket.htonl / ntohl / htons / ntohs /
# inet_aton / inet_ntoa byte-order helpers, select.POLLIN /
# POLLOUT / POLLERR / POLLHUP / POLLNVAL + select.select / poll,
# weakref.ref / WeakValueDictionary / WeakSet / WeakKeyDictionary
# class identity + ref() round-trip, pickle.DEFAULT_PROTOCOL /
# dumps protocol-marker bytes, pickle.PickleError / PicklingError
# / UnpicklingError / Pickler / Unpickler class identity,
# gc.garbage container surface all DIVERGE on mamba — moved to
# the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_socket_gc_pickle_value_ops {sum(_ledger)} asserts")
