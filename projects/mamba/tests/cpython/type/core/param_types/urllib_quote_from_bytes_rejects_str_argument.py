# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "param_types"
# dimension = "type"
# case = "urllib_quote_from_bytes_rejects_str_argument"
# subject = "urllib.parse.quote_from_bytes"
# kind = "mechanical"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""urllib.parse.quote_from_bytes: urllib_quote_from_bytes_rejects_str_argument (errors)."""
import urllib.parse

try:
    result = urllib.parse.quote_from_bytes("abc")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
