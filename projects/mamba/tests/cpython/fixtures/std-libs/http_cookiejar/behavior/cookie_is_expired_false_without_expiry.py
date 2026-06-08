# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookie_is_expired_false_without_expiry"
# subject = "http.cookiejar.Cookie"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.Cookie: Cookie.is_expired() is False for a cookie constructed with expires=None"""
import http.cookiejar

_c = http.cookiejar.Cookie(
    version=0, name="session", value="abc",
    port=None, port_specified=False,
    domain="example.com", domain_specified=True, domain_initial_dot=True,
    path="/", path_specified=True,
    secure=False, expires=None, discard=True,
    comment=None, comment_url=None, rest={},
)
assert not _c.is_expired(), "unexpired cookie"

print("cookie_is_expired_false_without_expiry OK")
