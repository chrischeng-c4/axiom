# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "surface"
# case = "cookieerror_exists"
# subject = "cookies"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies: cookieerror_exists (surface)."""
from http import cookies

assert hasattr(cookies, "CookieError")
print("cookieerror_exists OK")
