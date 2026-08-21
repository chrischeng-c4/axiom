# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "output_emits_set_cookie_line"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.SimpleCookie: output() serializes a cookie to a 'Set-Cookie: key=value' header line"""
from http import cookies

c = cookies.SimpleCookie()
c["session"] = "s123"
out = c.output()
assert out.startswith("Set-Cookie:"), f"output starts with Set-Cookie: {out!r}"
assert "session=s123" in out, f"session=s123 in output: {out!r}"
print("output_emits_set_cookie_line OK")
