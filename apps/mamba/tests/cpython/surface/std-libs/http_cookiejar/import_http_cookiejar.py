# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "surface"
# case = "import_http_cookiejar"
# subject = "http.cookiejar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.cookiejar: import_http_cookiejar (surface)."""
import http.cookiejar

assert hasattr(http.cookiejar, "CookieJar")
print("import_http_cookiejar OK")
