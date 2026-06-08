# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "domain_return_ok_decides_send"
# subject = "http.cookiejar.DefaultCookiePolicy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.DefaultCookiePolicy: DefaultCookiePolicy.domain_return_ok decides whether a domain's cookies would be sent back to a given request URL"""
import urllib.request
from http.cookiejar import DefaultCookiePolicy

_pol = DefaultCookiePolicy()
for url, domain, ok in [
    ("http://foo.bar.com/", "blah.com", False),
    ("http://foo.bar.com/", ".foo.bar.com", True),
    ("http://foo.bar.com/", ".bar.com", True),
    ("http://foo.bar.com/", "com", True),
    ("http://foo.com/", "rhubarb.foo.com", False),
    ("http://barfoo.com", ".foo.com", False),
]:
    _req = urllib.request.Request(url)
    assert bool(_pol.domain_return_ok(domain, _req)) == ok, (url, domain)

print("domain_return_ok_decides_send OK")
