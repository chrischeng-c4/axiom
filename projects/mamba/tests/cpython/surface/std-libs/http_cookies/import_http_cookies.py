# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "surface"
# case = "import_http_cookies"
# subject = "cookies.SimpleCookie"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.SimpleCookie: import_http_cookies (surface)."""
from http import cookies

assert hasattr(cookies.SimpleCookie, "load")
print("import_http_cookies OK")
