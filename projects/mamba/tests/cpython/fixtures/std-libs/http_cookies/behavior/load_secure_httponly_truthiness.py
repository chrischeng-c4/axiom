# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "load_secure_httponly_truthiness"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound load(); attributes are not populated (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: secure/httponly are falsy when absent from a loaded cookie and keep their explicit string value when present"""
from http import cookies

c = cookies.SimpleCookie()
c.load("eggs=scrambled; Path=/bacon")
assert not c["eggs"]["httponly"], "httponly absent is falsy"
assert not c["eggs"]["secure"], "secure absent is falsy"
c2 = cookies.SimpleCookie()
c2.load("eggs=scrambled; httponly=foo; secure=bar; Path=/bacon")
assert c2["eggs"]["httponly"] == "foo", f"httponly value = {c2['eggs']['httponly']!r}"
assert c2["eggs"]["secure"] == "bar", f"secure value = {c2['eggs']['secure']!r}"
print("load_secure_httponly_truthiness OK")
