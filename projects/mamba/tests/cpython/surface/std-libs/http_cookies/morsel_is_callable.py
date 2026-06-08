# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "surface"
# case = "morsel_is_callable"
# subject = "cookies.Morsel"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.Morsel: morsel_is_callable (surface)."""
from http import cookies

assert callable(cookies.Morsel)
print("morsel_is_callable OK")
