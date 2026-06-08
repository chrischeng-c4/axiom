# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "surface"
# case = "parse_headers_is_callable"
# subject = "http.client.parse_headers"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.parse_headers: parse_headers_is_callable (surface)."""
import http.client

assert callable(http.client.parse_headers)
print("parse_headers_is_callable OK")
