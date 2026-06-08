# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_outputstring_has_no_header"
# subject = "cookies.Morsel"
# kind = "semantic"
# xfail = "mamba Morsel shell has no bound OutputString() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.Morsel: Morsel.OutputString() returns 'key=value' without the leading 'Set-Cookie:' header"""
from http import cookies

c = cookies.SimpleCookie()
c["item"] = "value"
morsel = c["item"]
os = morsel.OutputString()
assert isinstance(os, str), f"OutputString type = {type(os)!r}"
assert "item=value" in os, f"OutputString has item=value: {os!r}"
assert "Set-Cookie:" not in os, f"OutputString lacks Set-Cookie: {os!r}"
print("morsel_outputstring_has_no_header OK")
