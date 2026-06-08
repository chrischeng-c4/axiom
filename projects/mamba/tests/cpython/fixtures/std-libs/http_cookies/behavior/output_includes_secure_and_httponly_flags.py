# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "output_includes_secure_and_httponly_flags"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie/Morsel shells do not render flag tokens in output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.SimpleCookie: secure and httponly Morsel flags render as valueless 'Secure' / 'HttpOnly' tokens in output()"""
from http import cookies

c = cookies.SimpleCookie()
c["secure_cookie"] = "val"
c["secure_cookie"]["secure"] = True
c["secure_cookie"]["httponly"] = True
out = c.output()
assert "Secure" in out, f"Secure in output: {out!r}"
assert "HttpOnly" in out, f"HttpOnly in output: {out!r}"
print("output_includes_secure_and_httponly_flags OK")
