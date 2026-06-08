# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "user_domain_match_netscape_rule"
# subject = "http.cookiejar.user_domain_match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.user_domain_match: user_domain_match implements the looser Netscape rule for user-supplied domains (exact host, dotted-suffix subdomain, never empty/'.')"""
from http.cookiejar import user_domain_match

assert user_domain_match("acme.com", "acme.com")
assert not user_domain_match("acme.com", ".acme.com")
assert user_domain_match("rhubarb.acme.com", ".acme.com")
assert user_domain_match("y.com", "Y.com")
assert not user_domain_match(".y.com", "Y.com")
assert user_domain_match("x.y.com", ".com")
assert not user_domain_match("x.y.com", "com")
assert not user_domain_match("x.y.com", "")
assert not user_domain_match("x.y.com", ".")
assert user_domain_match("192.168.1.1", "192.168.1.1")
assert not user_domain_match("192.168.1.1", ".168.1.1")

print("user_domain_match_netscape_rule OK")
