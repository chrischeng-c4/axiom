# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "unquote_plus_is_callable"
# subject = "urllib.parse.unquote_plus"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.unquote_plus: unquote_plus_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.unquote_plus)
print("unquote_plus_is_callable OK")
