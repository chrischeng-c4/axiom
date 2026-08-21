# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "urlunparse_is_callable"
# subject = "urllib.parse.urlunparse"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.urlunparse: urlunparse_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.urlunparse)
print("urlunparse_is_callable OK")
