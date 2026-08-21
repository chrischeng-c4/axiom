# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_socket_ssl_ipaddress_http_urllib_silent"
# subject = "cpython321.lang_socket_ssl_ipaddress_http_urllib_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_socket_ssl_ipaddress_http_urllib_silent.py"
# status = "filled"
# ///
"""cpython321.lang_socket_ssl_ipaddress_http_urllib_silent: execute CPython 3.12 seed lang_socket_ssl_ipaddress_http_urllib_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences across
# the `socket` module integer-constant / helper /
# exception / kind-class identifier surface +
# `socket.AF_INET6` integer-constant value contract +
# `socket.htons` byte-order helper value contract +
# `ssl.SSLCertVerificationError` exception identifier +
# `ssl.CERT_NONE` instance class-identity contract +
# `ipaddress.ip_address` / `ipaddress.ip_network`
# instance class-identity + string-projection
# contract + `http.HTTPMethod` enum identifier +
# `http.HTTPStatus.OK` enum-member-attribute /
# class-identity contract +
# `urllib.parse.ParseResult` class-identity contract
# pinned by atomic 209: `socket` (the documented
# integer-constant / helper / exception / kind-class
# identifier surface — `SOCK_RAW` / `SOL_SOCKET` /
# `SO_REUSEADDR` / `SO_KEEPALIVE` / `IPPROTO_TCP` /
# `IPPROTO_UDP` / `IPPROTO_IP` / `INADDR_ANY` /
# `INADDR_LOOPBACK` / `getnameinfo` /
# `getservbyname` / `getfqdn` / `has_dualstack_ipv6`
# / `if_nameindex` / `if_nametoindex` /
# `if_indextoname` / `ntohs` / `ntohl` / `htons` /
# `htonl` / `inet_aton` / `inet_ntoa` / `inet_pton`
# / `inet_ntop` / `error` / `herror` / `gaierror` /
# `timeout` / `AddressFamily` / `SocketKind` + the
# documented `socket.AF_INET6 == 30` integer-constant
# value contract — mamba collapses to 10 + the
# documented `socket.htons(80) == 20480` byte-order
# helper value contract — mamba: AttributeError),
# `ssl` (the documented
# `SSLCertVerificationError` exception identifier +
# the documented `type(ssl.CERT_NONE).__name__ ==
# "VerifyMode"` instance class-identity contract —
# mamba: "int"), `ipaddress` (the documented
# `type(ipaddress.ip_address("192.168.1.1"))
# .__name__ == "IPv4Address"` /
# `str(ipaddress.ip_address("192.168.1.1")) ==
# "192.168.1.1"` /
# `type(ipaddress.ip_network("192.168.1.0/24"))
# .__name__ == "IPv4Network"` /
# `str(ipaddress.ip_network("192.168.1.0/24")) ==
# "192.168.1.0/24"` instance class-identity +
# string-projection contract — mamba: collapsed to
# bare integer), `http` (the documented
# `HTTPMethod` enum identifier + the documented
# `http.HTTPStatus.OK.value == 200` /
# `http.HTTPStatus.NOT_FOUND.value == 404` /
# `type(http.HTTPStatus.OK).__name__ ==
# "HTTPStatus"` enum-member-attribute +
# class-identity contract — mamba: collapsed to
# bare integer), and `urllib.parse` (the documented
# `type(urllib.parse.urlparse(...)).__name__ ==
# "ParseResult"` class-identity contract — mamba
# leaks "urllib.parse.ParseResult").
#
# The matching subset (partial socket hasattr +
# integer-constant value contract, partial ssl
# hasattr + integer-constant value contract, full
# ipaddress hasattr, partial http hasattr +
# enum-member-equality value contract, urllib.parse
# helper value contract) is covered by
# `test_socket_ssl_ipaddress_http_urllib_value_ops`;
# this fixture pins the CPython-only contracts that
# mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • hasattr(socket, "SOCK_RAW") is True —
#     documented kind-sentinel identifier (mamba: False);
#   • hasattr(socket, "SOL_SOCKET") is True —
#     documented level-sentinel identifier (mamba: False);
#   • hasattr(socket, "SO_REUSEADDR") is True —
#     documented option-sentinel identifier (mamba: False);
#   • hasattr(socket, "SO_KEEPALIVE") is True —
#     documented option-sentinel identifier (mamba: False);
#   • hasattr(socket, "IPPROTO_TCP") is True —
#     documented protocol-sentinel identifier (mamba: False);
#   • hasattr(socket, "IPPROTO_UDP") is True —
#     documented protocol-sentinel identifier (mamba: False);
#   • hasattr(socket, "IPPROTO_IP") is True —
#     documented protocol-sentinel identifier (mamba: False);
#   • hasattr(socket, "INADDR_ANY") is True —
#     documented address-sentinel identifier (mamba: False);
#   • hasattr(socket, "INADDR_LOOPBACK") is True —
#     documented address-sentinel identifier (mamba: False);
#   • hasattr(socket, "getnameinfo") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(socket, "getservbyname") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(socket, "getfqdn") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(socket, "has_dualstack_ipv6") is True —
#     documented capability identifier (mamba: False);
#   • hasattr(socket, "if_nameindex") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(socket, "if_nametoindex") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(socket, "if_indextoname") is True —
#     documented helper identifier (mamba: False);
#   • hasattr(socket, "ntohs") is True — documented
#     byte-order helper identifier (mamba: False);
#   • hasattr(socket, "ntohl") is True — documented
#     byte-order helper identifier (mamba: False);
#   • hasattr(socket, "htons") is True — documented
#     byte-order helper identifier (mamba: False);
#   • hasattr(socket, "htonl") is True — documented
#     byte-order helper identifier (mamba: False);
#   • hasattr(socket, "inet_aton") is True —
#     documented address-conversion identifier (mamba: False);
#   • hasattr(socket, "inet_ntoa") is True —
#     documented address-conversion identifier (mamba: False);
#   • hasattr(socket, "inet_pton") is True —
#     documented address-conversion identifier (mamba: False);
#   • hasattr(socket, "inet_ntop") is True —
#     documented address-conversion identifier (mamba: False);
#   • hasattr(socket, "error") is True — documented
#     exception identifier (mamba: False);
#   • hasattr(socket, "herror") is True — documented
#     exception identifier (mamba: False);
#   • hasattr(socket, "gaierror") is True — documented
#     exception identifier (mamba: False);
#   • hasattr(socket, "timeout") is True — documented
#     exception identifier (mamba: False);
#   • hasattr(socket, "AddressFamily") is True —
#     documented enum-class identifier (mamba: False);
#   • hasattr(socket, "SocketKind") is True —
#     documented enum-class identifier (mamba: False);
#   • socket.AF_INET6 == 30 — documented
#     integer-constant value (mamba: 10);
#   • socket.htons(80) == 20480 — documented
#     byte-order helper value (mamba: AttributeError);
#   • hasattr(ssl, "SSLCertVerificationError") is True
#     — documented exception identifier (mamba: False);
#   • type(ssl.CERT_NONE).__name__ == "VerifyMode" —
#     documented instance class-identity (mamba: "int");
#   • type(ipaddress.ip_address("192.168.1.1"))
#     .__name__ == "IPv4Address" — documented
#     instance class-identity (mamba: "int");
#   • str(ipaddress.ip_address("192.168.1.1")) ==
#     "192.168.1.1" — documented
#     string-projection value (mamba: bare integer);
#   • type(ipaddress.ip_network("192.168.1.0/24"))
#     .__name__ == "IPv4Network" — documented
#     instance class-identity (mamba: "int");
#   • str(ipaddress.ip_network("192.168.1.0/24")) ==
#     "192.168.1.0/24" — documented
#     string-projection value (mamba: bare integer);
#   • hasattr(http, "HTTPMethod") is True —
#     documented enum-class identifier (mamba: False);
#   • http.HTTPStatus.OK.value == 200 — documented
#     enum-member-attribute value (mamba: None);
#   • http.HTTPStatus.NOT_FOUND.value == 404 —
#     documented enum-member-attribute value
#     (mamba: None);
#   • type(http.HTTPStatus.OK).__name__ ==
#     "HTTPStatus" — documented instance
#     class-identity (mamba: "int");
#   • type(urllib.parse.urlparse(
#     "http://www.example.com/path?q=1")).__name__ ==
#     "ParseResult" — documented class-identity
#     (mamba: "urllib.parse.ParseResult").
import socket as _socket_mod
import ssl as _ssl_mod
import ipaddress as _ipaddress_mod
import http as _http_mod
import urllib.parse as _urllib_parse_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identity / module-attribute / instance-method /
# string-projection identifier behavior that mamba's bundled
# type stubs do not surface accurately.
socket: Any = _socket_mod
ssl: Any = _ssl_mod
ipaddress: Any = _ipaddress_mod
http: Any = _http_mod
urllib_parse: Any = _urllib_parse_mod


_ledger: list[int] = []

# 1) socket — kind-sentinel / option-sentinel / protocol-sentinel /
#    address-sentinel / helper / byte-order helper /
#    address-conversion / exception / enum-class identifier surface
assert hasattr(socket, "SOCK_RAW") == True; _ledger.append(1)
assert hasattr(socket, "SOL_SOCKET") == True; _ledger.append(1)
assert hasattr(socket, "SO_REUSEADDR") == True; _ledger.append(1)
assert hasattr(socket, "SO_KEEPALIVE") == True; _ledger.append(1)
assert hasattr(socket, "IPPROTO_TCP") == True; _ledger.append(1)
assert hasattr(socket, "IPPROTO_UDP") == True; _ledger.append(1)
assert hasattr(socket, "IPPROTO_IP") == True; _ledger.append(1)
assert hasattr(socket, "INADDR_ANY") == True; _ledger.append(1)
assert hasattr(socket, "INADDR_LOOPBACK") == True; _ledger.append(1)
assert hasattr(socket, "getnameinfo") == True; _ledger.append(1)
assert hasattr(socket, "getservbyname") == True; _ledger.append(1)
assert hasattr(socket, "getfqdn") == True; _ledger.append(1)
assert hasattr(socket, "has_dualstack_ipv6") == True; _ledger.append(1)
assert hasattr(socket, "if_nameindex") == True; _ledger.append(1)
assert hasattr(socket, "if_nametoindex") == True; _ledger.append(1)
assert hasattr(socket, "if_indextoname") == True; _ledger.append(1)
assert hasattr(socket, "ntohs") == True; _ledger.append(1)
assert hasattr(socket, "ntohl") == True; _ledger.append(1)
assert hasattr(socket, "htons") == True; _ledger.append(1)
assert hasattr(socket, "htonl") == True; _ledger.append(1)
assert hasattr(socket, "inet_aton") == True; _ledger.append(1)
assert hasattr(socket, "inet_ntoa") == True; _ledger.append(1)
assert hasattr(socket, "inet_pton") == True; _ledger.append(1)
assert hasattr(socket, "inet_ntop") == True; _ledger.append(1)
assert hasattr(socket, "error") == True; _ledger.append(1)
assert hasattr(socket, "herror") == True; _ledger.append(1)
assert hasattr(socket, "gaierror") == True; _ledger.append(1)
assert hasattr(socket, "timeout") == True; _ledger.append(1)
assert hasattr(socket, "AddressFamily") == True; _ledger.append(1)
assert hasattr(socket, "SocketKind") == True; _ledger.append(1)

# 2) socket — integer-constant + byte-order helper value contract
assert socket.AF_INET6 == 30; _ledger.append(1)
assert socket.htons(80) == 20480; _ledger.append(1)

# 3) ssl — exception identifier + instance class-identity contract
assert hasattr(ssl, "SSLCertVerificationError") == True; _ledger.append(1)
assert type(ssl.CERT_NONE).__name__ == "VerifyMode"; _ledger.append(1)

# 4) ipaddress — instance class-identity + string-projection contract
_addr = ipaddress.ip_address("192.168.1.1")
assert type(_addr).__name__ == "IPv4Address"; _ledger.append(1)
assert str(_addr) == "192.168.1.1"; _ledger.append(1)
_net = ipaddress.ip_network("192.168.1.0/24")
assert type(_net).__name__ == "IPv4Network"; _ledger.append(1)
assert str(_net) == "192.168.1.0/24"; _ledger.append(1)

# 5) http — enum identifier + enum-member-attribute / class-identity contract
assert hasattr(http, "HTTPMethod") == True; _ledger.append(1)
assert http.HTTPStatus.OK.value == 200; _ledger.append(1)
assert http.HTTPStatus.NOT_FOUND.value == 404; _ledger.append(1)
assert type(http.HTTPStatus.OK).__name__ == "HTTPStatus"; _ledger.append(1)

# 6) urllib.parse — class-identity contract
_p = urllib_parse.urlparse("http://www.example.com/path?q=1")
assert type(_p).__name__ == "ParseResult"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_socket_ssl_ipaddress_http_urllib_silent {sum(_ledger)} asserts")
