# Operational AssertionPass divergence-spec fixture for the silent
# value-contract divergences of `hasattr(socketserver, 'BaseServer')`
# (the documented "socketserver exposes the BaseServer class" — mamba
# returns False), `hasattr(socketserver, 'TCPServer')` (the documented
# "socketserver exposes the TCPServer class" — mamba returns False),
# `hasattr(socketserver, 'UDPServer')` (the documented "socketserver
# exposes the UDPServer class" — mamba returns False), `hasattr(
# socketserver, 'ThreadingMixIn')` (the documented "socketserver
# exposes the ThreadingMixIn class" — mamba returns False), `hasattr(
# string, 'printable')` (the documented "string exposes the printable
# constant" — mamba returns False), `type(string.Template('$x')).
# __name__ == 'Template'` (the documented "string.Template(s)
# constructs a Template instance" — mamba returns 'dict' — constructor
# degrades to plain dict), `type(string.Formatter()).__name__ ==
# 'Formatter'` (the documented "string.Formatter() constructs a
# Formatter instance" — mamba returns 'dict' — constructor degrades to
# plain dict), `hasattr(sched, 'scheduler')` (the documented "sched
# exposes the scheduler class" — mamba returns False — module body
# resolves to None), `hasattr(sched, 'Event')` (the documented "sched
# exposes the Event named-tuple" — mamba returns False — module body
# resolves to None), and `type(_thread.allocate_lock()).__name__ ==
# 'lock'` (the documented "_thread.allocate_lock() returns a lock
# instance with the lowercase 'lock' type name" — mamba returns 'dict'
# — constructor degrades to plain dict).
# Ten-pack pinned to atomic 311.
#
# Behavioral edges that CONFORM on mamba (ssl — hasattr SSLContext/
# SSLSocket/SSLError/CERT_NONE/CERT_OPTIONAL/CERT_REQUIRED/PROTOCOL_
# TLS/PROTOCOL_TLS_CLIENT/PROTOCOL_TLS_SERVER/Purpose/create_default_
# context/OP_ALL/OP_NO_SSLv2/OP_NO_SSLv3/VERIFY_DEFAULT + CERT_NONE ==
# 0 + CERT_OPTIONAL == 1 + CERT_REQUIRED == 2. string — hasattr ascii
# _letters/ascii_lowercase/ascii_uppercase/digits/hexdigits/octdigits/
# punctuation/whitespace/Formatter/Template/capwords + ascii_lowercase
# + digits + hexdigits + octdigits + capwords. _thread — hasattr
# start_new_thread/allocate_lock/get_ident/LockType/exit/error/
# interrupt_main + isinstance(get_ident(), int). stringprep — hasattr
# in_table_a1/in_table_b1/in_table_c11/in_table_c12/in_table_c21/in_
# table_c22/in_table_d1/in_table_d2 + in_table_a1('a') False + in_
# table_c11(' ') True) are covered in the matching pass fixture
# `test_ssl_string_thread_stringprep_value_ops`.
import socketserver
import string
import sched
import _thread


_ledger: list[int] = []

# 1) hasattr(socketserver, 'BaseServer') — BaseServer class
#    (mamba: returns False)
assert hasattr(socketserver, "BaseServer") == True; _ledger.append(1)

# 2) hasattr(socketserver, 'TCPServer') — TCPServer class
#    (mamba: returns False)
assert hasattr(socketserver, "TCPServer") == True; _ledger.append(1)

# 3) hasattr(socketserver, 'UDPServer') — UDPServer class
#    (mamba: returns False)
assert hasattr(socketserver, "UDPServer") == True; _ledger.append(1)

# 4) hasattr(socketserver, 'ThreadingMixIn') — ThreadingMixIn class
#    (mamba: returns False)
assert hasattr(socketserver, "ThreadingMixIn") == True; _ledger.append(1)

# 5) hasattr(string, 'printable') — printable constant
#    (mamba: returns False)
assert hasattr(string, "printable") == True; _ledger.append(1)

# 6) type(string.Template('$x')).__name__ == 'Template' — Template instance
#    (mamba: returns 'dict' — constructor degrades to plain dict)
assert type(string.Template("$x")).__name__ == "Template"; _ledger.append(1)

# 7) type(string.Formatter()).__name__ == 'Formatter' — Formatter instance
#    (mamba: returns 'dict' — constructor degrades to plain dict)
assert type(string.Formatter()).__name__ == "Formatter"; _ledger.append(1)

# 8) hasattr(sched, 'scheduler') — scheduler class
#    (mamba: returns False — module body resolves to None)
assert hasattr(sched, "scheduler") == True; _ledger.append(1)

# 9) hasattr(sched, 'Event') — Event named-tuple
#    (mamba: returns False — module body resolves to None)
assert hasattr(sched, "Event") == True; _ledger.append(1)

# 10) type(_thread.allocate_lock()).__name__ == 'lock' — lock instance lowercase type name
#     (mamba: returns 'dict' — constructor degrades to plain dict)
assert type(_thread.allocate_lock()).__name__ == "lock"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_socketserver_string_sched_silent {sum(_ledger)} asserts")
