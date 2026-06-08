# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "domain_match_rfc2965"
# subject = "http.cookiejar.domain_match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.domain_match: domain_match is case-insensitive; a leading-dot pattern matches the domain and its subdomains while IP literals match only exactly"""
from http.cookiejar import domain_match

assert domain_match("192.168.1.1", "192.168.1.1")
assert not domain_match("192.168.1.1", ".168.1.1")
assert domain_match("x.y.com", "x.Y.com")
assert domain_match("x.y.com", ".Y.com")
assert not domain_match("x.y.com", "Y.com")
assert domain_match("a.b.c.com", ".c.com")
assert not domain_match(".c.com", "a.b.c.com")
assert domain_match("example.local", ".local")
assert not domain_match("blah.blah", "")
assert domain_match("", "")

print("domain_match_rfc2965 OK")
