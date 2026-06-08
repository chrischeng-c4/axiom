# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "load_collapses_surrounding_whitespace"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound load()/output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: load() collapses surrounding whitespace around keys/values/attrs; output() is the canonical compact form"""
from http import cookies

c = cookies.SimpleCookie()
c.load("eggs  =  scrambled  ;  secure  ;  path  =  bar   ; foo=foo   ")
assert c.output() == "Set-Cookie: eggs=scrambled; Path=bar; Secure\r\nSet-Cookie: foo=foo", \
    f"extra-space output = {c.output()!r}"
print("load_collapses_surrounding_whitespace OK")
