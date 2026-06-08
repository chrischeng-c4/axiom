# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "behavior"
# case = "ipv6_scope_id_and_userinfo"
# subject = "urllib.parse.urlsplit"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urlparse.py"
# status = "filled"
# ///
"""urllib.parse.urlsplit: an RFC 6874 scope id ('%zone') stays on the lowercased hostname while netloc preserves original case; userinfo + bracketed host + port parse together"""
from urllib.parse import urlsplit

sc = urlsplit("http://[FE80::822a:a8ff:fe49:470c%tESt]:1234")
assert sc.hostname == "fe80::822a:a8ff:fe49:470c%tESt", f"scoped hostname = {sc.hostname!r}"
assert sc.netloc == "[FE80::822a:a8ff:fe49:470c%tESt]:1234", f"netloc preserves case = {sc.netloc!r}"

u = urlsplit("scheme://user@[v6a.ip]:1234/path?query")
assert u.username == "user", f"user = {u.username!r}"
assert u.hostname == "v6a.ip", f"host = {u.hostname!r}"
assert u.port == 1234, f"port = {u.port!r}"
assert u.path == "/path", f"path = {u.path!r}"

print("ipv6_scope_id_and_userinfo OK")
