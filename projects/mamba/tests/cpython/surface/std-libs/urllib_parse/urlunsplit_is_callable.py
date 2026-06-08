# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "urlunsplit_is_callable"
# subject = "urllib.parse.urlunsplit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.urlunsplit: urlunsplit_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.urlunsplit)
print("urlunsplit_is_callable OK")
