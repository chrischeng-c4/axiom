# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "load_parses_cookie_header"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound load(); no parsing (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.SimpleCookie: load() parses a 'k1=v1; k2=v2; k3=v3' Cookie header into one Morsel per key"""
from http import cookies

c = cookies.SimpleCookie()
c.load("name=John; age=30; city=NYC")
assert c["name"].value == "John", f"name = {c['name'].value!r}"
assert c["age"].value == "30", f"age = {c['age'].value!r}"
assert c["city"].value == "NYC", f"city = {c['city'].value!r}"
print("load_parses_cookie_header OK")
