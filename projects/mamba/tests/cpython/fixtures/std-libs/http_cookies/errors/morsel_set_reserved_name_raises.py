# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "errors"
# case = "morsel_set_reserved_name_raises"
# subject = "cookies.Morsel"
# kind = "mechanical"
# xfail = "mamba Morsel shell has no bound set(); reserved-name guard is absent (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.Morsel: morsel_set_reserved_name_raises (errors)."""
from http import cookies

_raised = False
try:
    cookies.Morsel().set('expires', 'v', 'v')
except cookies.CookieError:
    _raised = True
assert _raised, "morsel_set_reserved_name_raises: expected cookies.CookieError"
print("morsel_set_reserved_name_raises OK")
