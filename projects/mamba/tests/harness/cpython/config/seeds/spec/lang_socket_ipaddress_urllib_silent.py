# Operational AssertionPass seed for SILENT divergences across
# the `socket` extended constant / function / exception
# identifier surface + `ipaddress.ip_address` instance class
# identity contract + `urllib.parse` dotted-submodule attr
# surface + `urllib.error` dotted-submodule attr surface +
# `urllib.request` dotted-submodule attr surface + the
# `urlparse(...)` instance class identity contract pinned by
# atomic 198: `socket` (the documented `SOCK_RAW` /
# `SOL_SOCKET` / `SO_REUSEADDR` / `SO_KEEPALIVE` /
# `gethostbyaddr` / `timeout` / `error` / `herror` /
# `gaierror` / `socketpair` / `fromfd` / `htonl` / `htons` /
# `ntohl` / `ntohs` / `inet_aton` / `inet_ntoa` /
# `inet_pton` / `inet_ntop` / `IPPROTO_TCP` / `IPPROTO_UDP`
# extended constant / function / exception identifier
# surface), `ipaddress.ip_address` (the documented
# `IPv4Address` instance class identity contract — mamba
# collapses to "int" via the integer-handle pattern),
# `urllib.parse` (the documented `urlparse` / `urlunparse`
# / `urlsplit` / `urlunsplit` / `urljoin` / `urlencode` /
# `parse_qs` / `parse_qsl` / `quote` / `quote_plus` /
# `quote_from_bytes` / `unquote` / `unquote_plus` /
# `unquote_to_bytes` / `urldefrag` / `ParseResult` /
# `SplitResult` dotted-submodule attr surface — mamba
# elides via the dotted-submodule-attr-empty quirk),
# `urllib.error` (the documented `URLError` / `HTTPError`
# / `ContentTooShortError` dotted-submodule attr surface),
# `urllib.request` (the documented `urlopen` / `Request` /
# `build_opener` / `install_opener` / `HTTPHandler` /
# `HTTPSHandler` / `HTTPDefaultErrorHandler` /
# `HTTPRedirectHandler` / `HTTPCookieProcessor` /
# `ProxyHandler` / `BaseHandler` / `FileHandler` /
# `FTPHandler` / `url2pathname` / `pathname2url` dotted-
# submodule attr surface), and `urlparse(...)` (the
# documented `ParseResult` named-tuple class identity —
# mamba leaks the module-qualified `urllib.parse
# .ParseResult` name).
#
# The matching subset (partial socket hasattr + AF_INET /
# SOCK_STREAM / SOCK_DGRAM integer values, full ipaddress
# hasattr, urllib.parse value contract via `from urllib
# .parse import X` for urlparse / urljoin / quote /
# unquote / urlencode) is covered by
# `test_socket_ipaddress_urllib_parse_value_ops`; this
# fixture pins the CPython-only contracts that mamba
# currently elides.
import socket as _socket_mod
import ipaddress as _ipaddress_mod
import urllib.parse as _urllib_parse_mod
import urllib.error as _urllib_error_mod
import urllib.request as _urllib_request_mod
from urllib.parse import urlparse
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# constant / function / class / dotted-submodule attribute /
# class-identity behavior that mamba's bundled type stubs do
# not surface accurately.
socket: Any = _socket_mod
ipaddress: Any = _ipaddress_mod
urllib_parse: Any = _urllib_parse_mod
urllib_error: Any = _urllib_error_mod
urllib_request: Any = _urllib_request_mod


_ledger: list[int] = []

# 1) socket — extended constant / function / exception surface
assert hasattr(socket, "SOCK_RAW") == True; _ledger.append(1)
assert hasattr(socket, "SOL_SOCKET") == True; _ledger.append(1)
assert hasattr(socket, "SO_REUSEADDR") == True; _ledger.append(1)
assert hasattr(socket, "SO_KEEPALIVE") == True; _ledger.append(1)
assert hasattr(socket, "gethostbyaddr") == True; _ledger.append(1)
assert hasattr(socket, "timeout") == True; _ledger.append(1)
assert hasattr(socket, "error") == True; _ledger.append(1)
assert hasattr(socket, "herror") == True; _ledger.append(1)
assert hasattr(socket, "gaierror") == True; _ledger.append(1)
assert hasattr(socket, "socketpair") == True; _ledger.append(1)
assert hasattr(socket, "fromfd") == True; _ledger.append(1)
assert hasattr(socket, "htonl") == True; _ledger.append(1)
assert hasattr(socket, "htons") == True; _ledger.append(1)
assert hasattr(socket, "ntohl") == True; _ledger.append(1)
assert hasattr(socket, "ntohs") == True; _ledger.append(1)
assert hasattr(socket, "inet_aton") == True; _ledger.append(1)
assert hasattr(socket, "inet_ntoa") == True; _ledger.append(1)
assert hasattr(socket, "inet_pton") == True; _ledger.append(1)
assert hasattr(socket, "inet_ntop") == True; _ledger.append(1)
assert hasattr(socket, "IPPROTO_TCP") == True; _ledger.append(1)
assert hasattr(socket, "IPPROTO_UDP") == True; _ledger.append(1)

# 2) ipaddress.ip_address — instance class identity contract
assert type(ipaddress.ip_address("127.0.0.1")).__name__ == "IPv4Address"; _ledger.append(1)

# 3) urllib.parse — dotted-submodule attr surface
assert hasattr(urllib_parse, "urlparse") == True; _ledger.append(1)
assert hasattr(urllib_parse, "urlunparse") == True; _ledger.append(1)
assert hasattr(urllib_parse, "urlsplit") == True; _ledger.append(1)
assert hasattr(urllib_parse, "urlunsplit") == True; _ledger.append(1)
assert hasattr(urllib_parse, "urljoin") == True; _ledger.append(1)
assert hasattr(urllib_parse, "urlencode") == True; _ledger.append(1)
assert hasattr(urllib_parse, "parse_qs") == True; _ledger.append(1)
assert hasattr(urllib_parse, "parse_qsl") == True; _ledger.append(1)
assert hasattr(urllib_parse, "quote") == True; _ledger.append(1)
assert hasattr(urllib_parse, "quote_plus") == True; _ledger.append(1)
assert hasattr(urllib_parse, "quote_from_bytes") == True; _ledger.append(1)
assert hasattr(urllib_parse, "unquote") == True; _ledger.append(1)
assert hasattr(urllib_parse, "unquote_plus") == True; _ledger.append(1)
assert hasattr(urllib_parse, "unquote_to_bytes") == True; _ledger.append(1)
assert hasattr(urllib_parse, "urldefrag") == True; _ledger.append(1)
assert hasattr(urllib_parse, "ParseResult") == True; _ledger.append(1)
assert hasattr(urllib_parse, "SplitResult") == True; _ledger.append(1)

# 4) urllib.error — dotted-submodule attr surface
assert hasattr(urllib_error, "URLError") == True; _ledger.append(1)
assert hasattr(urllib_error, "HTTPError") == True; _ledger.append(1)
assert hasattr(urllib_error, "ContentTooShortError") == True; _ledger.append(1)

# 5) urllib.request — dotted-submodule attr surface
assert hasattr(urllib_request, "urlopen") == True; _ledger.append(1)
assert hasattr(urllib_request, "Request") == True; _ledger.append(1)
assert hasattr(urllib_request, "build_opener") == True; _ledger.append(1)
assert hasattr(urllib_request, "install_opener") == True; _ledger.append(1)
assert hasattr(urllib_request, "HTTPHandler") == True; _ledger.append(1)
assert hasattr(urllib_request, "HTTPSHandler") == True; _ledger.append(1)
assert hasattr(urllib_request, "HTTPDefaultErrorHandler") == True; _ledger.append(1)
assert hasattr(urllib_request, "HTTPRedirectHandler") == True; _ledger.append(1)
assert hasattr(urllib_request, "HTTPCookieProcessor") == True; _ledger.append(1)
assert hasattr(urllib_request, "ProxyHandler") == True; _ledger.append(1)
assert hasattr(urllib_request, "BaseHandler") == True; _ledger.append(1)
assert hasattr(urllib_request, "FileHandler") == True; _ledger.append(1)
assert hasattr(urllib_request, "FTPHandler") == True; _ledger.append(1)
assert hasattr(urllib_request, "url2pathname") == True; _ledger.append(1)
assert hasattr(urllib_request, "pathname2url") == True; _ledger.append(1)

# 6) urlparse — instance class identity contract
assert type(urlparse("https://example.com")).__name__ == "ParseResult"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_socket_ipaddress_urllib_silent {sum(_ledger)} asserts")
