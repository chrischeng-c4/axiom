# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookiejar_clear_session_cookies"
# subject = "http.cookiejar.CookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.CookieJar: clear_session_cookies() removes discard=True (no-expiry session) cookies, emptying a jar that holds only session cookies"""
import http.cookiejar


def _make_cookie(name, value, domain="example.com", path="/", expires=None, secure=False):
    return http.cookiejar.Cookie(
        version=0, name=name, value=value,
        port=None, port_specified=False,
        domain=domain, domain_specified=True, domain_initial_dot=True,
        path=path, path_specified=True,
        secure=secure, expires=expires, discard=True,
        comment=None, comment_url=None, rest={},
    )


_jar = http.cookiejar.CookieJar()
_jar.set_cookie(_make_cookie("session", "abc", expires=None))  # discard=True
_jar.clear_session_cookies()
assert len(_jar) == 0, f"session cookies cleared: {len(_jar)!r}"

print("cookiejar_clear_session_cookies OK")
