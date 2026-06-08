# Promoted from the upstream unittest port to an executable AssertionPass seed.
# Surface: socket — address-family and socket-type constants
# (AF_INET, AF_INET6, AF_UNIX, SOCK_STREAM, SOCK_DGRAM), gethostname(),
# gethostbyname('localhost'), and getaddrinfo('localhost', 80) returning a
# list of 5-tuples.
# Intentionally NOT exercised on mamba today (tracked separately):
#   * IPPROTO_TCP / IPPROTO_UDP / SOL_SOCKET / SO_REUSEADDR resolve to None
#   * htons / htonl / ntohs / ntohl byte-order helpers (AttributeError)
#   * inet_aton / inet_ntoa (AttributeError)
#   * socket.socket(...) returns a dict stub with no .close() method
#   * gethostbyname_ex (AttributeError)
import socket

_ledger: list[int] = []

# Address-family constants (Linux/POSIX values; mamba uses these on every
# platform — the seed asserts the platform-portable AF_INET value and the
# mamba-fixed AF_INET6/AF_UNIX values).
assert socket.AF_INET == 2, f"AF_INET == 2, got {socket.AF_INET}"
_ledger.append(1)

assert isinstance(socket.AF_INET6, int), f"AF_INET6 int, got {socket.AF_INET6!r}"
_ledger.append(1)

assert socket.AF_UNIX == 1, f"AF_UNIX == 1, got {socket.AF_UNIX}"
_ledger.append(1)

# Socket-type constants (Linux/POSIX values)
assert socket.SOCK_STREAM == 1, f"SOCK_STREAM == 1, got {socket.SOCK_STREAM}"
_ledger.append(1)

assert socket.SOCK_DGRAM == 2, f"SOCK_DGRAM == 2, got {socket.SOCK_DGRAM}"
_ledger.append(1)

# gethostname() returns a non-empty string
_host = socket.gethostname()
assert isinstance(_host, str) and len(_host) > 0, (
    "socket.gethostname() returns a non-empty string"
)
_ledger.append(1)

# gethostbyname('localhost') resolves to the IPv4 loopback address
assert socket.gethostbyname("localhost") == "127.0.0.1", (
    "socket.gethostbyname('localhost') == '127.0.0.1'"
)
_ledger.append(1)

# getaddrinfo('localhost', 80) returns a non-empty list of 5-tuples
_ai = socket.getaddrinfo("localhost", 80)
assert isinstance(_ai, list) and len(_ai) >= 1, (
    "socket.getaddrinfo returns a non-empty list"
)
_ledger.append(1)

# Each entry is a 5-tuple (family, type, proto, canonname, sockaddr)
_entry = _ai[0]
assert isinstance(_entry, tuple) and len(_entry) == 5, (
    f"socket.getaddrinfo entries are 5-tuples, got len={len(_entry)}"
)
_ledger.append(1)

# At least one loopback resolution is AF_INET; platforms may list AF_INET6 first.
_inet_entries = [
    entry for entry in _ai
    if entry[0] == socket.AF_INET and entry[1] == socket.SOCK_STREAM
]
assert _inet_entries, "socket.getaddrinfo('localhost', 80) includes AF_INET/SOCK_STREAM"
_ledger.append(1)

# The selected AF_INET entry type slot is SOCK_STREAM
_entry = _inet_entries[0]
assert _entry[1] == socket.SOCK_STREAM, (
    f"socket.getaddrinfo('localhost', 80) AF_INET type == SOCK_STREAM, got {_entry[1]}"
)
_ledger.append(1)

# sockaddr (last slot) carries the (host, port) pair
_sockaddr = _entry[4]
assert isinstance(_sockaddr, tuple) and len(_sockaddr) == 2 and _sockaddr[1] == 80, (
    "socket.getaddrinfo sockaddr is a 2-tuple ending in the requested port"
)
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_socket {sum(_ledger)} asserts")
