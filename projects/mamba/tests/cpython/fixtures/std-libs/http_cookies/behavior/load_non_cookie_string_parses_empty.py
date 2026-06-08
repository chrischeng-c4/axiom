# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "load_non_cookie_string_parses_empty"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound load()/output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: a non-cookie string (Set-Cookie:, version-only, leading bare flag) parses to an empty cookie with empty output()"""
from http import cookies

c = cookies.SimpleCookie()
for bad in ("Set-Cookie: foo=bar", "foo=bar; baz", "secure;foo=bar", "Version=1;foo=bar"):
    c.load(bad)
    assert dict(c) == {}, f"non-cookie {bad!r} parsed empty"
    assert c.output() == "", f"non-cookie {bad!r} empty output"
print("load_non_cookie_string_parses_empty OK")
