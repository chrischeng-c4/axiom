# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "output_flags_alphabetical_order"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie/Morsel shells do not render ordered flag tokens in output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: setting secure+httponly emits valueless flags in alphabetical order: '; HttpOnly; Secure'"""
from http import cookies

c = cookies.SimpleCookie('Customer="WILE_E_COYOTE"')
c["Customer"]["secure"] = True
c["Customer"]["httponly"] = True
assert c.output() == 'Set-Cookie: Customer="WILE_E_COYOTE"; HttpOnly; Secure', \
    f"flag output = {c.output()!r}"
print("output_flags_alphabetical_order OK")
