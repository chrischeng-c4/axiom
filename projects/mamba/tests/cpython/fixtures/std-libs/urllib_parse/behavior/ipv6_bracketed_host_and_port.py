# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "ipv6_bracketed_host_and_port"
# subject = "urllib.parse.urlsplit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: RFC 2732 brackets are stripped from .hostname (lowercased) with the trailing ':port' parsed: '[dead:beef::1]:5432' gives hostname 'dead:beef::1' port 5432, and '[::1]' alone gives port None"""
from urllib.parse import urlsplit

p = urlsplit("http://[dead:beef::1]:5432/foo/")
assert p.hostname == "dead:beef::1", f"hostname = {p.hostname!r}"
assert p.port == 5432, f"port = {p.port!r}"

p2 = urlsplit("http://[dead:BEEF:cafe::12.34.56.78]/foo/")
assert p2.hostname == "dead:beef:cafe::12.34.56.78", f"hostname lowercased = {p2.hostname!r}"
assert p2.port is None, f"no port = {p2.port!r}"

p3 = urlsplit("http://[::1]/foo/")
assert p3.hostname == "::1", f"hostname = {p3.hostname!r}"
assert p3.port is None, f"port = {p3.port!r}"

print("ipv6_bracketed_host_and_port OK")
