# Operational AssertionPass seed for SILENT divergences across the
# networking / select / weakref / serialization quartet pinned by
# atomic 144: `socket` (the AF_INET6 / SOCK_RAW / SOL_SOCKET /
# SO_REUSEADDR / SO_KEEPALIVE / IPPROTO_TCP / IPPROTO_UDP /
# SHUT_RD / SHUT_WR / SHUT_RDWR documented integer sentinels +
# the socket / error / gaierror class identity + htonl / ntohl /
# htons / ntohs / inet_aton / inet_ntoa byte-order helpers),
# `select` (POLLIN / POLLOUT / POLLERR / POLLHUP / POLLNVAL
# poll-event bitflags + the select / poll module-level helpers),
# `weakref` (the ref / proxy / WeakValueDictionary / WeakSet /
# WeakKeyDictionary class identity plus the documented round-trip
# contract `weakref.ref(c)() is c`), and `pickle` (the
# DEFAULT_PROTOCOL == 4 PEP 3154 sentinel on CPython 3.12, the
# `b'\\x80\\x04'` protocol-marker byte prefix of `pickle.dumps`,
# the PickleError / PicklingError / UnpicklingError / Pickler /
# Unpickler class identity).
#
# The matching subset (socket.AF_INET / AF_UNIX / SOCK_STREAM /
# SOCK_DGRAM integer sentinels, gc.isenabled / get_count /
# get_threshold / collect / get_objects / DEBUG_* bitflags,
# pickle.HIGHEST_PROTOCOL == 5 + lossless atomic-type and
# container round-trip) is covered by
# `test_socket_gc_pickle_value_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • socket.AF_INET6 == 30 — macOS BSD IPv6 family sentinel
#     (mamba: returns 10, the Linux glibc value — the platform-
#     conditional constant was pinned to the Linux build);
#   • socket.SOCK_RAW == 3 — POSIX raw-socket type sentinel
#     (mamba: returns None);
#   • socket.SOL_SOCKET == 65535 — macOS BSD socket-level sentinel
#     (mamba: returns None);
#   • socket.SO_REUSEADDR == 4 (mamba: None);
#   • socket.SO_KEEPALIVE == 8 (mamba: None);
#   • socket.IPPROTO_TCP == 6 — IANA TCP protocol number
#     (mamba: None);
#   • socket.IPPROTO_UDP == 17 — IANA UDP protocol number
#     (mamba: None);
#   • socket.SHUT_RD == 0, SHUT_WR == 1, SHUT_RDWR == 2 — POSIX
#     shutdown-direction sentinels (mamba: all None);
#   • socket.socket.__name__ == "socket" — bare class identity
#     (mamba: returns None — socket.socket is a lambda);
#   • socket.error is OSError — documented OSError alias (mamba:
#     returns None);
#   • socket.gaierror.__name__ == "gaierror" — DNS-resolver
#     error class (mamba: None);
#   • socket.htonl(1) == 16777216 — host-to-network long
#     byte-swap (mamba: AttributeError, socket is a dict);
#   • socket.inet_aton("1.2.3.4") == b"\\x01\\x02\\x03\\x04"
#     (mamba: AttributeError);
#   • select.POLLIN == 1, POLLOUT == 4, POLLERR == 8, POLLHUP
#     == 16, POLLNVAL == 32 — poll-event bitflags (mamba: all
#     None);
#   • hasattr(select, "select") is True, hasattr(select, "poll")
#     is True — module-level helpers (mamba: both False);
#   • weakref.ref(_obj)() is _obj — round-trip contract (mamba:
#     returns None — weakref is broken);
#   • weakref.ref.__name__ == "ReferenceType" (mamba: None);
#   • weakref.WeakValueDictionary.__name__ ==
#     "WeakValueDictionary" (mamba: None);
#   • weakref.WeakSet.__name__ == "WeakSet" (mamba: None);
#   • weakref.WeakKeyDictionary.__name__ == "WeakKeyDictionary"
#     (mamba: None);
#   • pickle.DEFAULT_PROTOCOL == 4 — PEP 3154 default on
#     CPython 3.12 (mamba: returns 5);
#   • pickle.dumps([1, 2, 3])[:2] == b"\\x80\\x04" — protocol-
#     marker prefix (mamba: returns b"L3" — mamba uses an
#     entirely different on-the-wire encoding);
#   • pickle.PickleError.__name__ == "PickleError" (mamba: None);
#   • pickle.PicklingError.__name__ == "PicklingError" (mamba:
#     None);
#   • pickle.UnpicklingError.__name__ == "UnpicklingError"
#     (mamba: None);
#   • pickle.Pickler.__name__ == "Pickler" (mamba: None);
#   • pickle.Unpickler.__name__ == "Unpickler" (mamba: None);
#   • type(gc.garbage).__name__ == "list" — uncollectable-object
#     container (mamba: returns "NoneType", the binding is
#     missing).
import socket as _socket_mod
import select as _select_mod
import weakref as _weakref_mod
import pickle as _pickle_mod
import gc as _gc_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# constants / class identifiers / module-level helpers that
# mamba's bundled type stubs do not surface accurately.
socket: Any = _socket_mod
select: Any = _select_mod
weakref: Any = _weakref_mod
pickle: Any = _pickle_mod
gc: Any = _gc_mod


# Helper class kept at module scope to dodge the documented mamba
# quirk where a `class` defined inside a `try:` block isn't
# visible to the next statement.
class _Anchor:
    pass


_ledger: list[int] = []

# 1) socket.AF_INET6 — macOS BSD IPv6 family sentinel
assert socket.AF_INET6 == 30; _ledger.append(1)

# 2) socket — POSIX socket-type sentinels (raw)
assert socket.SOCK_RAW == 3; _ledger.append(1)

# 3) socket — BSD socket-level / option sentinels
assert socket.SOL_SOCKET == 65535; _ledger.append(1)
assert socket.SO_REUSEADDR == 4; _ledger.append(1)
assert socket.SO_KEEPALIVE == 8; _ledger.append(1)

# 4) socket — IANA protocol-number sentinels
assert socket.IPPROTO_TCP == 6; _ledger.append(1)
assert socket.IPPROTO_UDP == 17; _ledger.append(1)

# 5) socket — POSIX shutdown-direction sentinels
assert socket.SHUT_RD == 0; _ledger.append(1)
assert socket.SHUT_WR == 1; _ledger.append(1)
assert socket.SHUT_RDWR == 2; _ledger.append(1)

# 6) socket — class identity surface
assert socket.socket.__name__ == "socket"; _ledger.append(1)
assert socket.error is OSError; _ledger.append(1)
assert socket.gaierror.__name__ == "gaierror"; _ledger.append(1)

# 7) socket — byte-order / address-format helpers
assert socket.htonl(1) == 16777216; _ledger.append(1)
assert socket.ntohl(16777216) == 1; _ledger.append(1)
assert socket.inet_aton("1.2.3.4") == b"\x01\x02\x03\x04"; _ledger.append(1)

# 8) select — poll-event bitflags
assert select.POLLIN == 1; _ledger.append(1)
assert select.POLLOUT == 4; _ledger.append(1)
assert select.POLLERR == 8; _ledger.append(1)
assert select.POLLHUP == 16; _ledger.append(1)
assert select.POLLNVAL == 32; _ledger.append(1)

# 9) select — module-level helpers
assert hasattr(select, "select") == True; _ledger.append(1)
assert hasattr(select, "poll") == True; _ledger.append(1)

# 10) weakref — round-trip contract
_obj = _Anchor()
_ref = weakref.ref(_obj)
assert _ref() is _obj; _ledger.append(1)

# 11) weakref — class-identity surface
assert weakref.ref.__name__ == "ReferenceType"; _ledger.append(1)
assert weakref.WeakValueDictionary.__name__ == "WeakValueDictionary"; _ledger.append(1)
assert weakref.WeakSet.__name__ == "WeakSet"; _ledger.append(1)
assert weakref.WeakKeyDictionary.__name__ == "WeakKeyDictionary"; _ledger.append(1)

# 12) pickle — PEP 3154 default-protocol sentinel
assert pickle.DEFAULT_PROTOCOL == 4; _ledger.append(1)

# 13) pickle — dumps protocol-marker byte prefix
assert pickle.dumps([1, 2, 3])[:2] == b"\x80\x04"; _ledger.append(1)

# 14) pickle — class-identity surface
assert pickle.PickleError.__name__ == "PickleError"; _ledger.append(1)
assert pickle.PicklingError.__name__ == "PicklingError"; _ledger.append(1)
assert pickle.UnpicklingError.__name__ == "UnpicklingError"; _ledger.append(1)
assert pickle.Pickler.__name__ == "Pickler"; _ledger.append(1)
assert pickle.Unpickler.__name__ == "Unpickler"; _ledger.append(1)

# 15) gc.garbage — uncollectable-object container surface
assert type(gc.garbage).__name__ == "list"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_socket_select_weakref_pickle_silent {sum(_ledger)} asserts")
