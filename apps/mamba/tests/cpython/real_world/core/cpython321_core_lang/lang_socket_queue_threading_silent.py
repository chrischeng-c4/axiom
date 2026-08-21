# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_socket_queue_threading_silent"
# subject = "cpython321.lang_socket_queue_threading_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_socket_queue_threading_silent.py"
# status = "filled"
# ///
"""cpython321.lang_socket_queue_threading_silent: execute CPython 3.12 seed lang_socket_queue_threading_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(socket, 'SOCK_RAW')` (the
# documented "socket exposes the SOCK_RAW socket-type constant" —
# mamba returns False), `hasattr(socket, 'SOL_SOCKET')` (the
# documented "socket exposes the SOL_SOCKET option-level constant"
# — mamba returns False), `hasattr(socket, 'htons')` (the
# documented "socket exposes the htons host->network short helper"
# — mamba returns False), `hasattr(socket, 'inet_aton')` (the
# documented "socket exposes the inet_aton dotted-quad parser" —
# mamba returns False), `hasattr(socket, 'error')` (the documented
# "socket exposes the socket.error exception alias" — mamba
# returns False), `hasattr(select, 'select')` (the documented
# "select exposes the select(rl, wl, xl) syscall wrapper" —
# mamba returns False), `hasattr(select, 'poll')` (the documented
# "select exposes the poll() factory on platforms that support it"
# — mamba returns False), `hasattr(select, 'PIPE_BUF')` (the
# documented "select exposes the PIPE_BUF kernel pipe-atomicity
# constant" — mamba returns False), `type(queue.Queue()).__name__`
# (the documented "queue.Queue() returns a Queue instance" — mamba
# returns 'int' — Queue() yields an int handle), and `type
# (threading.current_thread()).__name__` (the documented "the main
# thread's class is _MainThread" — mamba returns 'Thread').
# Ten-pack pinned to atomic 275.
#
# Behavioral edges that CONFORM on mamba (socket — hasattr socket/
# AF_INET/AF_INET6/AF_UNIX/SOCK_STREAM/SOCK_DGRAM/gethostname/
# gethostbyname + AF_INET/SOCK_STREAM are int + gethostname is
# str. queue — hasattr Queue/LifoQueue/PriorityQueue/SimpleQueue/
# Empty/Full + Queue FIFO get/put + Queue empty + qsize +
# LifoQueue LIFO. threading — full hasattr surface + active_count/
# get_ident int + get_ident>0) are covered in the matching pass
# fixture `test_socket_queue_threading_value_ops`.
import socket
import select
import queue
import threading


_ledger: list[int] = []

# 1) hasattr(socket, 'SOCK_RAW') — raw-socket type
#    (mamba: returns False)
assert hasattr(socket, "SOCK_RAW") == True; _ledger.append(1)

# 2) hasattr(socket, 'SOL_SOCKET') — option-level constant
#    (mamba: returns False)
assert hasattr(socket, "SOL_SOCKET") == True; _ledger.append(1)

# 3) hasattr(socket, 'htons') — byte-order helper
#    (mamba: returns False)
assert hasattr(socket, "htons") == True; _ledger.append(1)

# 4) hasattr(socket, 'inet_aton') — dotted-quad parser
#    (mamba: returns False)
assert hasattr(socket, "inet_aton") == True; _ledger.append(1)

# 5) hasattr(socket, 'error') — socket exception alias
#    (mamba: returns False)
assert hasattr(socket, "error") == True; _ledger.append(1)

# 6) hasattr(select, 'select') — select() syscall wrapper
#    (mamba: returns False)
assert hasattr(select, "select") == True; _ledger.append(1)

# 7) hasattr(select, 'poll') — poll() factory
#    (mamba: returns False)
assert hasattr(select, "poll") == True; _ledger.append(1)

# 8) hasattr(select, 'PIPE_BUF') — pipe-atomicity constant
#    (mamba: returns False)
assert hasattr(select, "PIPE_BUF") == True; _ledger.append(1)

# 9) type(queue.Queue()).__name__ == 'Queue' — instance type
#    (mamba: returns 'int' — Queue() yields an int handle)
assert type(queue.Queue()).__name__ == "Queue"; _ledger.append(1)

# 10) type(threading.current_thread()).__name__ == '_MainThread'
#     (mamba: returns 'Thread' — no _MainThread subclass)
assert type(threading.current_thread()).__name__ == "_MainThread"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_socket_queue_threading_silent {sum(_ledger)} asserts")
