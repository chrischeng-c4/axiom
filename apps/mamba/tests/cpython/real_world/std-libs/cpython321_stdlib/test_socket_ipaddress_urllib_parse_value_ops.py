# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_socket_ipaddress_urllib_parse_value_ops"
# subject = "cpython321.test_socket_ipaddress_urllib_parse_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_socket_ipaddress_urllib_parse_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_socket_ipaddress_urllib_parse_value_ops: execute CPython 3.12 seed test_socket_ipaddress_urllib_parse_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of the
# `socket` / `ipaddress` / `urllib.parse` three-pack pinned to
# atomic 198: `socket` (the documented partial module-level
# constant / function identifier hasattr surface — `socket`
# / `AF_INET` / `AF_INET6` / `AF_UNIX` / `SOCK_STREAM` /
# `SOCK_DGRAM` / `gethostname` / `gethostbyname` /
# `getaddrinfo` / `create_connection` / `create_server` + the
# documented socket.AF_INET == 2 / SOCK_STREAM == 1 /
# SOCK_DGRAM == 2 integer-value contract), `ipaddress` (the
# documented full module-level helper / class identifier
# hasattr surface — `ip_address` / `ip_network` /
# `ip_interface` / `IPv4Address` / `IPv4Network` /
# `IPv4Interface` / `IPv6Address` / `IPv6Network` /
# `IPv6Interface` / `AddressValueError` /
# `NetmaskValueError` / `summarize_address_range` /
# `collapse_addresses` / `get_mixed_type_key`), and
# `urllib.parse` (the documented value contract via
# `from urllib.parse import X` for the documented public
# helper surface — `urlparse` / `urljoin` / `quote` /
# `unquote` / `urlencode`).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(socket, "SOCK_RAW") / "SOL_SOCKET" /
# "SO_REUSEADDR" / "SO_KEEPALIVE" / "gethostbyaddr" /
# "timeout" / "error" / "herror" / "gaierror" /
# "socketpair" / "fromfd" / "htonl" / "htons" / "ntohl" /
# "ntohs" / "inet_aton" / "inet_ntoa" / "inet_pton" /
# "inet_ntop" / "IPPROTO_TCP" / "IPPROTO_UDP" all False on
# mamba, type(ipaddress.ip_address("127.0.0.1")).__name__
# collapses to "int" on mamba via the integer-handle
# pattern, hasattr(urllib.parse, "urlparse") / "urljoin" /
# "quote" / "unquote" / "urlencode" / "ParseResult" all
# False on mamba via the dotted-submodule-attr-empty quirk,
# hasattr(urllib.request, "urlopen") / "Request" /
# "build_opener" all False on mamba, hasattr(urllib.error,
# "URLError") / "HTTPError" all False on mamba,
# type(urlparse(...)).__name__ collapses to
# "urllib.parse.ParseResult" on mamba via the module-
# qualified type-name leak) are covered in the matching spec
# fixture `lang_socket_ipaddress_urllib_silent`.
import socket
import ipaddress
from urllib.parse import urlparse, urljoin, quote, unquote, urlencode


_ledger: list[int] = []

# 1) socket — partial module hasattr surface
#    (SOCK_RAW / SOL_SOCKET / SO_REUSEADDR / SO_KEEPALIVE /
#    gethostbyaddr / timeout / error / herror / gaierror /
#    socketpair / fromfd / htonl / htons / ntohl / ntohs /
#    inet_aton / inet_ntoa / inet_pton / inet_ntop /
#    IPPROTO_TCP / IPPROTO_UDP DIVERGE — moved to spec)
assert hasattr(socket, "socket") == True; _ledger.append(1)
assert hasattr(socket, "AF_INET") == True; _ledger.append(1)
assert hasattr(socket, "AF_INET6") == True; _ledger.append(1)
assert hasattr(socket, "AF_UNIX") == True; _ledger.append(1)
assert hasattr(socket, "SOCK_STREAM") == True; _ledger.append(1)
assert hasattr(socket, "SOCK_DGRAM") == True; _ledger.append(1)
assert hasattr(socket, "gethostname") == True; _ledger.append(1)
assert hasattr(socket, "gethostbyname") == True; _ledger.append(1)
assert hasattr(socket, "getaddrinfo") == True; _ledger.append(1)
assert hasattr(socket, "create_connection") == True; _ledger.append(1)
assert hasattr(socket, "create_server") == True; _ledger.append(1)

# 2) socket — integer-value contract
assert socket.AF_INET == 2; _ledger.append(1)
assert socket.SOCK_STREAM == 1; _ledger.append(1)
assert socket.SOCK_DGRAM == 2; _ledger.append(1)

# 3) ipaddress — full module hasattr surface
#    (instance class identity collapses to "int" via the
#    integer-handle pattern — moved to spec fixture)
assert hasattr(ipaddress, "ip_address") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_network") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_interface") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv4Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv4Network") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv4Interface") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Network") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Interface") == True; _ledger.append(1)
assert hasattr(ipaddress, "AddressValueError") == True; _ledger.append(1)
assert hasattr(ipaddress, "NetmaskValueError") == True; _ledger.append(1)
assert hasattr(ipaddress, "summarize_address_range") == True; _ledger.append(1)
assert hasattr(ipaddress, "collapse_addresses") == True; _ledger.append(1)
assert hasattr(ipaddress, "get_mixed_type_key") == True; _ledger.append(1)

# 4) urllib.parse — value contract via `from` import
#    (dotted-submodule-attr access DIVERGES — moved to spec)
_r = urlparse("https://example.com/path?q=1")
assert _r.scheme == "https"; _ledger.append(1)
assert _r.netloc == "example.com"; _ledger.append(1)
assert _r.path == "/path"; _ledger.append(1)
assert _r.query == "q=1"; _ledger.append(1)
assert urljoin("https://example.com/a/b", "../c") == "https://example.com/c"; _ledger.append(1)
assert quote("hello world") == "hello%20world"; _ledger.append(1)
assert unquote("hello%20world") == "hello world"; _ledger.append(1)
assert urlencode({"a": 1, "b": 2}) == "a=1&b=2"; _ledger.append(1)

# NB: hasattr(socket, "SOCK_RAW") / "SOL_SOCKET" /
# "SO_REUSEADDR" / "SO_KEEPALIVE" / "gethostbyaddr" /
# "timeout" / "error" / "herror" / "gaierror" /
# "socketpair" / "fromfd" / "htonl" / "htons" / "ntohl" /
# "ntohs" / "inet_aton" / "inet_ntoa" / "inet_pton" /
# "inet_ntop" / "IPPROTO_TCP" / "IPPROTO_UDP" all False on
# mamba, type(ipaddress.ip_address("127.0.0.1")).__name__
# collapses to "int" on mamba via the integer-handle
# pattern, hasattr(urllib.parse, "urlparse") / "urljoin" /
# "quote" / "unquote" / "urlencode" / "ParseResult" all
# False on mamba via the dotted-submodule-attr-empty quirk,
# hasattr(urllib.request, "urlopen") / "Request" /
# "build_opener" all False on mamba, hasattr(urllib.error,
# "URLError") / "HTTPError" all False on mamba,
# type(urlparse(...)).__name__ collapses to
# "urllib.parse.ParseResult" on mamba via the module-
# qualified type-name leak — all DIVERGE on mamba — moved
# to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_socket_ipaddress_urllib_parse_value_ops {sum(_ledger)} asserts")
