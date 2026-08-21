# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "parse_qsl_is_callable"
# subject = "urllib.parse.parse_qsl"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.parse_qsl: parse_qsl_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.parse_qsl)
print("parse_qsl_is_callable OK")
