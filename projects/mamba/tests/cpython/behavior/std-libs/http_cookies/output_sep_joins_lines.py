# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "output_sep_joins_lines"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: output(sep=...) joins multiple Set-Cookie lines with the chosen separator"""
from http import cookies

c = cookies.SimpleCookie()
c.load("chips=ahoy; vienna=finger")
assert c.output(sep="\n") == "Set-Cookie: chips=ahoy\nSet-Cookie: vienna=finger", \
    f"sep output = {c.output(sep=chr(10))!r}"
print("output_sep_joins_lines OK")
