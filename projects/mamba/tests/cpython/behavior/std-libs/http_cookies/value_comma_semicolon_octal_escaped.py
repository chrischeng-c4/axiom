# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "value_comma_semicolon_octal_escaped"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie/Morsel shells do not octal-escape delimiters in output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: comma and semicolon inside a value become \\054 and \\073 inside the quoted output()"""
from http import cookies

c = cookies.SimpleCookie()
c["val"] = "some,funky;stuff"
assert c.output(["val"]) == 'Set-Cookie: val="some\\054funky\\073stuff"', \
    f"extended = {c.output(['val'])!r}"
print("value_comma_semicolon_octal_escaped OK")
