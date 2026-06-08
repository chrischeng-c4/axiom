# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "output_max_age_and_expires"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie/Morsel shells do not render Max-Age/Expires tokens in output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: max-age renders as 'Max-Age=N' and expires=0 renders an absolute GMT date in output()"""
from http import cookies

c = cookies.SimpleCookie('Customer="WILE_E_COYOTE"')
c["Customer"]["max-age"] = 10
assert c.output() == 'Set-Cookie: Customer="WILE_E_COYOTE"; Max-Age=10', \
    f"max-age output = {c.output()!r}"
c2 = cookies.SimpleCookie('Customer="WILE_E_COYOTE"')
c2["Customer"]["expires"] = 0
assert c2.output().endswith("GMT"), f"expires output = {c2.output()!r}"
print("output_max_age_and_expires OK")
