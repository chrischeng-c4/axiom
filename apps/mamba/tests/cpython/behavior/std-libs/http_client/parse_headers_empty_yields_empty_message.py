# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "parse_headers_empty_yields_empty_message"
# subject = "http.client.parse_headers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.parse_headers: parse_headers on an empty byte stream returns an HTTPMessage with no header fields"""
import http.client as hc
import io

msg = hc.parse_headers(io.BytesIO(b""))
assert type(msg).__name__ == "HTTPMessage", f"type = {type(msg).__name__}"
assert list(msg.keys()) == [], f"keys = {list(msg.keys())!r}"

print("parse_headers_empty_yields_empty_message OK")
