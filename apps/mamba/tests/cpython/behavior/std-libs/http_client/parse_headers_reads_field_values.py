# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_client"
# dimension = "behavior"
# case = "parse_headers_reads_field_values"
# subject = "http.client.parse_headers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""http.client.parse_headers: parse_headers reads a CRLF-terminated header block into an HTTPMessage whose case-insensitive lookup returns the field values"""
import http.client as hc
import io

block = b"Content-Type: text/html\r\nContent-Length: 42\r\n\r\n"
msg = hc.parse_headers(io.BytesIO(block))
# Case-insensitive field lookup, both styles.
assert msg.get("content-type") == "text/html", f"content-type = {msg.get('content-type')!r}"
assert msg["Content-Length"] == "42", f"Content-Length = {msg['Content-Length']!r}"
assert len(msg.keys()) == 2, f"keys = {list(msg.keys())!r}"

print("parse_headers_reads_field_values OK")
