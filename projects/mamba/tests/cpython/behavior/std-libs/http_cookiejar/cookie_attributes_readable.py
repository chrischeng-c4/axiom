# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookie_attributes_readable"
# subject = "http.cookiejar.Cookie"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.Cookie: a constructed Cookie exposes name/value/domain/path/secure/discard/expires exactly as supplied"""
import http.cookiejar

_c = http.cookiejar.Cookie(
    version=0, name="token", value="xyz",
    port=None, port_specified=False,
    domain=".example.com", domain_specified=True, domain_initial_dot=True,
    path="/api", path_specified=True,
    secure=True, expires=None, discard=True,
    comment=None, comment_url=None, rest={},
)
assert _c.name == "token", f"name = {_c.name!r}"
assert _c.value == "xyz", f"value = {_c.value!r}"
assert _c.domain == ".example.com", f"domain = {_c.domain!r}"
assert _c.path == "/api", f"path = {_c.path!r}"
assert _c.secure == True, f"secure = {_c.secure!r}"
assert _c.discard == True, f"discard = {_c.discard!r}"
assert _c.expires is None, f"expires = {_c.expires!r}"

print("cookie_attributes_readable OK")
