# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "urljoin_is_callable"
# subject = "urllib.parse.urljoin"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.urljoin: urljoin_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.urljoin)
print("urljoin_is_callable OK")
