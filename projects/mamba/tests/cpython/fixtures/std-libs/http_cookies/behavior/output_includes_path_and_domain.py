# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "output_includes_path_and_domain"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie/Morsel shells do not render attribute tokens in output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.SimpleCookie: Path and Domain Morsel attributes render as 'Path=...' / 'Domain=...' tokens in output()"""
from http import cookies

c = cookies.SimpleCookie()
c["auth"] = "token123"
c["auth"]["path"] = "/api"
c["auth"]["domain"] = ".example.com"
out = c.output()
assert "Path=/api" in out, f"Path in output: {out!r}"
assert "Domain=.example.com" in out, f"Domain in output: {out!r}"
print("output_includes_path_and_domain OK")
