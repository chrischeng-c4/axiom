# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "output_filtered_attrs"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: output(attrs) restricts the rendered Morsel attributes to the named subset"""
from http import cookies

c = cookies.SimpleCookie()
c.load('Customer="WILE_E_COYOTE"; Version=1; Path=/acme')
assert c.output(["path"]) == 'Set-Cookie: Customer="WILE_E_COYOTE"; Path=/acme', \
    f"filtered output = {c.output(['path'])!r}"
print("output_filtered_attrs OK")
