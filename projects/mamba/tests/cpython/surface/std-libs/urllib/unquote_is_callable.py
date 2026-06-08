# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "unquote_is_callable"
# subject = "urllib.parse.unquote"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.unquote: unquote_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.unquote)
print("unquote_is_callable OK")
