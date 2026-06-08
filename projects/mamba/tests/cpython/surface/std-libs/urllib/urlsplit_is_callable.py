# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "urlsplit_is_callable"
# subject = "urllib.parse.urlsplit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.urlsplit: urlsplit_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.urlsplit)
print("urlsplit_is_callable OK")
