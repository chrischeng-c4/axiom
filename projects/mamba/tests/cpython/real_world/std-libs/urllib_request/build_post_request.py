# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "real_world"
# case = "build_post_request"
# subject = "urllib.request.Request"
# kind = "semantic"
# xfail = "urllib.request unimplemented on mamba: Request() returns a dict (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = "Lib/test/test_urllib2.py"
# status = "filled"
# ///
"""urllib.request.Request: an HTTP-client wrapper builds a POST Request with a JSON body and headers, then reads back get_method()=='POST', the title-cased Content-Type header, and the parsed host/selector before dispatch (in-memory, no network)"""
from urllib.request import Request

# Build the request the way a REST-API client wrapper would, then inspect
# what it would put on the wire -- no network call is made.
body = b'{"name": "Alice", "age": 30}'
req = Request(
    "https://api.example.com/v1/users?role=admin",
    data=body,
    headers={"Content-Type": "application/json", "Accept": "application/json"},
    method="POST",
)

# method: explicit POST (also implied by the data body)
assert req.get_method() == "POST", f"method = {req.get_method()!r}"

# headers are title-cased on storage; both are readable back
assert req.has_header("Content-type"), "Content-Type header present"
assert req.get_header("Content-type") == "application/json", f"content-type = {req.get_header('Content-type')!r}"
assert req.has_header("Accept"), "Accept header present"

# the parsed URL the client would dispatch against
assert req.type == "https", f"scheme = {req.type!r}"
assert req.host == "api.example.com", f"host = {req.host!r}"
assert req.selector == "/v1/users?role=admin", f"selector = {req.selector!r}"

# the body is preserved byte-for-byte
assert req.data == body, f"data = {req.data!r}"

print("build_post_request OK")
