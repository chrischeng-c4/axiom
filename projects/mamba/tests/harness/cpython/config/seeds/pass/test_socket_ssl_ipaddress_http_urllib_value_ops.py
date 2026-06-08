# Operational AssertionPass seed for the value contract of the
# `socket` / `ssl` / `ipaddress` / `http` / `urllib.parse`
# five-pack pinned to atomic 209: `socket` (the documented
# partial module-level class / helper / address-family /
# socket-kind identifier hasattr surface — `socket` /
# `create_connection` / `create_server` / `AF_INET` /
# `AF_INET6` / `AF_UNIX` / `SOCK_STREAM` / `SOCK_DGRAM` /
# `gethostname` / `gethostbyname` / `getaddrinfo` + the
# documented `AF_INET == 2` / `SOCK_STREAM == 1` /
# `SOCK_DGRAM == 2` integer-constant value contract),
# `ssl` (the documented partial module-level class /
# exception / helper / protocol-sentinel /
# certificate-mode-sentinel / verification-mode-sentinel
# / openssl-introspection / capability-sentinel
# identifier hasattr surface — `SSLContext` /
# `SSLSocket` / `SSLError` / `SSLZeroReturnError` /
# `SSLWantReadError` / `SSLWantWriteError` /
# `SSLSyscallError` / `SSLEOFError` / `CertificateError`
# / `create_default_context` / `PROTOCOL_TLS` /
# `PROTOCOL_TLS_CLIENT` / `PROTOCOL_TLS_SERVER` /
# `CERT_NONE` / `CERT_OPTIONAL` / `CERT_REQUIRED` /
# `Purpose` / `VERIFY_DEFAULT` / `VERIFY_CRL_CHECK_LEAF`
# / `VERIFY_X509_STRICT` / `OPENSSL_VERSION` /
# `OPENSSL_VERSION_INFO` / `OPENSSL_VERSION_NUMBER` /
# `HAS_SNI` / `HAS_ECDH` / `HAS_NPN` / `HAS_ALPN` /
# `HAS_TLSv1` / `HAS_TLSv1_1` / `HAS_TLSv1_2` /
# `HAS_TLSv1_3` + the documented `CERT_NONE == 0` /
# `CERT_REQUIRED == 2` / `PROTOCOL_TLS_CLIENT == 16`
# integer-constant value contract), `ipaddress` (the
# documented full module-level class / helper /
# exception identifier hasattr surface —
# `IPv4Address` / `IPv6Address` / `IPv4Network` /
# `IPv6Network` / `IPv4Interface` / `IPv6Interface` /
# `ip_address` / `ip_network` / `ip_interface` /
# `collapse_addresses` / `summarize_address_range` /
# `get_mixed_type_key` / `AddressValueError` /
# `NetmaskValueError`), `http` (the documented
# partial module-level enum identifier hasattr
# surface — `HTTPStatus` + the documented
# `http.HTTPStatus.OK == 200` enum-member-equality
# value contract), and `urllib.parse` (the
# documented helper value contract —
# `urlparse("http://www.example.com/path?q=1")
# .scheme == "http"` / `...netloc == "www.example
# .com"` / `...path == "/path"` / `...query ==
# "q=1"` / `urlencode({"a": "1", "b": "2"}) ==
# "a=1&b=2"` / `quote("a b") == "a%20b"` /
# `unquote("a%20b") == "a b"`).
#
# Behavioral edges that DIVERGE on mamba
# (hasattr(socket, "SOCK_RAW") / "SOL_SOCKET" /
# "SO_REUSEADDR" / "SO_KEEPALIVE" / "IPPROTO_TCP" /
# "IPPROTO_UDP" / "IPPROTO_IP" / "INADDR_ANY" /
# "INADDR_LOOPBACK" / "getnameinfo" / "getservbyname"
# / "getfqdn" / "has_dualstack_ipv6" / "if_nameindex"
# / "if_nametoindex" / "if_indextoname" / "ntohs" /
# "ntohl" / "htons" / "htonl" / "inet_aton" /
# "inet_ntoa" / "inet_pton" / "inet_ntop" / "error" /
# "herror" / "gaierror" / "timeout" / "AddressFamily"
# / "SocketKind" all False on mamba + socket.AF_INET6
# == 30 collapses to 10 on mamba + socket.htons(80)
# == 20480 unavailable on mamba, hasattr(ssl,
# "SSLCertVerificationError") False on mamba +
# type(ssl.CERT_NONE).__name__ == "VerifyMode"
# collapses to "int" on mamba, type(ipaddress
# .ip_address("192.168.1.1")).__name__ ==
# "IPv4Address" collapses to "int" on mamba +
# str(ipaddress.ip_address("192.168.1.1")) ==
# "192.168.1.1" collapses to bare integer on mamba +
# type(ipaddress.ip_network(...)).__name__ ==
# "IPv4Network" collapses to "int" on mamba +
# str(ipaddress.ip_network(...)) == "192.168.1.0/24"
# collapses to bare integer on mamba, hasattr(http,
# "HTTPMethod") False on mamba + http.HTTPStatus.OK
# .value == 200 collapses to None on mamba +
# type(http.HTTPStatus.OK).__name__ == "HTTPStatus"
# collapses to "int" on mamba, type(urllib.parse
# .urlparse(...)).__name__ == "ParseResult" leaks to
# "urllib.parse.ParseResult" on mamba) are covered
# in the matching spec fixture
# `lang_socket_ssl_ipaddress_http_urllib_silent`.
import socket
import ssl
import ipaddress
import http
import urllib.parse as urllib_parse


_ledger: list[int] = []

# 1) socket — partial module hasattr surface
#    (SOCK_RAW / SOL_SOCKET / SO_REUSEADDR /
#    SO_KEEPALIVE / IPPROTO_TCP / IPPROTO_UDP /
#    IPPROTO_IP / INADDR_ANY / INADDR_LOOPBACK /
#    getnameinfo / getservbyname / getfqdn /
#    has_dualstack_ipv6 / if_nameindex /
#    if_nametoindex / if_indextoname / ntohs /
#    ntohl / htons / htonl / inet_aton /
#    inet_ntoa / inet_pton / inet_ntop / error /
#    herror / gaierror / timeout / AddressFamily
#    / SocketKind all DIVERGE on mamba — moved to
#    spec)
assert hasattr(socket, "socket") == True; _ledger.append(1)
assert hasattr(socket, "create_connection") == True; _ledger.append(1)
assert hasattr(socket, "create_server") == True; _ledger.append(1)
assert hasattr(socket, "AF_INET") == True; _ledger.append(1)
assert hasattr(socket, "AF_INET6") == True; _ledger.append(1)
assert hasattr(socket, "AF_UNIX") == True; _ledger.append(1)
assert hasattr(socket, "SOCK_STREAM") == True; _ledger.append(1)
assert hasattr(socket, "SOCK_DGRAM") == True; _ledger.append(1)
assert hasattr(socket, "gethostname") == True; _ledger.append(1)
assert hasattr(socket, "gethostbyname") == True; _ledger.append(1)
assert hasattr(socket, "getaddrinfo") == True; _ledger.append(1)

# 2) socket — integer-constant value contract
assert socket.AF_INET == 2; _ledger.append(1)
assert socket.SOCK_STREAM == 1; _ledger.append(1)
assert socket.SOCK_DGRAM == 2; _ledger.append(1)

# 3) ssl — partial module hasattr surface
#    (SSLCertVerificationError DIVERGES on mamba — moved to spec)
assert hasattr(ssl, "SSLContext") == True; _ledger.append(1)
assert hasattr(ssl, "SSLSocket") == True; _ledger.append(1)
assert hasattr(ssl, "SSLError") == True; _ledger.append(1)
assert hasattr(ssl, "SSLZeroReturnError") == True; _ledger.append(1)
assert hasattr(ssl, "SSLWantReadError") == True; _ledger.append(1)
assert hasattr(ssl, "SSLWantWriteError") == True; _ledger.append(1)
assert hasattr(ssl, "SSLSyscallError") == True; _ledger.append(1)
assert hasattr(ssl, "SSLEOFError") == True; _ledger.append(1)
assert hasattr(ssl, "CertificateError") == True; _ledger.append(1)
assert hasattr(ssl, "create_default_context") == True; _ledger.append(1)
assert hasattr(ssl, "PROTOCOL_TLS") == True; _ledger.append(1)
assert hasattr(ssl, "PROTOCOL_TLS_CLIENT") == True; _ledger.append(1)
assert hasattr(ssl, "PROTOCOL_TLS_SERVER") == True; _ledger.append(1)
assert hasattr(ssl, "CERT_NONE") == True; _ledger.append(1)
assert hasattr(ssl, "CERT_OPTIONAL") == True; _ledger.append(1)
assert hasattr(ssl, "CERT_REQUIRED") == True; _ledger.append(1)
assert hasattr(ssl, "Purpose") == True; _ledger.append(1)
assert hasattr(ssl, "VERIFY_DEFAULT") == True; _ledger.append(1)
assert hasattr(ssl, "VERIFY_CRL_CHECK_LEAF") == True; _ledger.append(1)
assert hasattr(ssl, "VERIFY_X509_STRICT") == True; _ledger.append(1)
assert hasattr(ssl, "OPENSSL_VERSION") == True; _ledger.append(1)
assert hasattr(ssl, "OPENSSL_VERSION_INFO") == True; _ledger.append(1)
assert hasattr(ssl, "OPENSSL_VERSION_NUMBER") == True; _ledger.append(1)
assert hasattr(ssl, "HAS_SNI") == True; _ledger.append(1)
assert hasattr(ssl, "HAS_ECDH") == True; _ledger.append(1)
assert hasattr(ssl, "HAS_NPN") == True; _ledger.append(1)
assert hasattr(ssl, "HAS_ALPN") == True; _ledger.append(1)
assert hasattr(ssl, "HAS_TLSv1") == True; _ledger.append(1)
assert hasattr(ssl, "HAS_TLSv1_1") == True; _ledger.append(1)
assert hasattr(ssl, "HAS_TLSv1_2") == True; _ledger.append(1)
assert hasattr(ssl, "HAS_TLSv1_3") == True; _ledger.append(1)

# 4) ssl — integer-constant value contract
assert ssl.CERT_NONE == 0; _ledger.append(1)
assert ssl.CERT_REQUIRED == 2; _ledger.append(1)
assert ssl.PROTOCOL_TLS_CLIENT == 16; _ledger.append(1)

# 5) ipaddress — full module hasattr surface
assert hasattr(ipaddress, "IPv4Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Address") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv4Network") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Network") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv4Interface") == True; _ledger.append(1)
assert hasattr(ipaddress, "IPv6Interface") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_address") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_network") == True; _ledger.append(1)
assert hasattr(ipaddress, "ip_interface") == True; _ledger.append(1)
assert hasattr(ipaddress, "collapse_addresses") == True; _ledger.append(1)
assert hasattr(ipaddress, "summarize_address_range") == True; _ledger.append(1)
assert hasattr(ipaddress, "get_mixed_type_key") == True; _ledger.append(1)
assert hasattr(ipaddress, "AddressValueError") == True; _ledger.append(1)
assert hasattr(ipaddress, "NetmaskValueError") == True; _ledger.append(1)

# 6) http — partial module hasattr surface
#    (HTTPMethod DIVERGES on mamba — moved to spec)
assert hasattr(http, "HTTPStatus") == True; _ledger.append(1)

# 7) http — HTTPStatus enum-member-equality value contract
assert http.HTTPStatus.OK == 200; _ledger.append(1)

# 8) urllib.parse — helper value contract
_p = urllib_parse.urlparse("http://www.example.com/path?q=1")
assert _p.scheme == "http"; _ledger.append(1)
assert _p.netloc == "www.example.com"; _ledger.append(1)
assert _p.path == "/path"; _ledger.append(1)
assert _p.query == "q=1"; _ledger.append(1)
assert urllib_parse.urlencode({"a": "1", "b": "2"}) == "a=1&b=2"; _ledger.append(1)
assert urllib_parse.quote("a b") == "a%20b"; _ledger.append(1)
assert urllib_parse.unquote("a%20b") == "a b"; _ledger.append(1)

# NB: hasattr(socket, "SOCK_RAW") / "SOL_SOCKET" /
# "SO_REUSEADDR" / "SO_KEEPALIVE" / "IPPROTO_TCP" /
# "IPPROTO_UDP" / "IPPROTO_IP" / "INADDR_ANY" /
# "INADDR_LOOPBACK" / "getnameinfo" / "getservbyname"
# / "getfqdn" / "has_dualstack_ipv6" / "if_nameindex"
# / "if_nametoindex" / "if_indextoname" / "ntohs" /
# "ntohl" / "htons" / "htonl" / "inet_aton" /
# "inet_ntoa" / "inet_pton" / "inet_ntop" / "error" /
# "herror" / "gaierror" / "timeout" / "AddressFamily"
# / "SocketKind" all False on mamba + socket.AF_INET6
# == 30 collapses to 10 on mamba + socket.htons(80)
# == 20480 unavailable on mamba, hasattr(ssl,
# "SSLCertVerificationError") False on mamba +
# type(ssl.CERT_NONE).__name__ == "VerifyMode"
# collapses to "int" on mamba, type(ipaddress
# .ip_address("192.168.1.1")).__name__ ==
# "IPv4Address" collapses to "int" on mamba +
# str(ipaddress.ip_address("192.168.1.1")) ==
# "192.168.1.1" collapses to bare integer on mamba +
# type(ipaddress.ip_network(...)).__name__ ==
# "IPv4Network" collapses to "int" on mamba +
# str(ipaddress.ip_network(...)) == "192.168.1.0/24"
# collapses to bare integer on mamba, hasattr(http,
# "HTTPMethod") False on mamba + http.HTTPStatus.OK
# .value == 200 collapses to None on mamba +
# type(http.HTTPStatus.OK).__name__ == "HTTPStatus"
# collapses to "int" on mamba, type(urllib.parse
# .urlparse(...)).__name__ == "ParseResult" leaks to
# "urllib.parse.ParseResult" on mamba — all DIVERGE
# on mamba — moved to the divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_socket_ssl_ipaddress_http_urllib_value_ops {sum(_ledger)} asserts")
