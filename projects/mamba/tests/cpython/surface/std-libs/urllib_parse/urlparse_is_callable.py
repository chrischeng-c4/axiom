# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "urlparse_is_callable"
# subject = "urllib.parse.urlparse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.urlparse: urlparse_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.urlparse)
print("urlparse_is_callable OK")
