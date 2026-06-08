# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "repr_shows_quoted_values"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound load() and a generic repr (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: repr(SimpleCookie) lists each key with its single-quoted value: <SimpleCookie: chips='ahoy' vienna='finger'>"""
from http import cookies

c = cookies.SimpleCookie()
c.load("chips=ahoy; vienna=finger")
assert repr(c) == "<SimpleCookie: chips='ahoy' vienna='finger'>", f"repr = {c!r}"
print("repr_shows_quoted_values OK")
