# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookiejar_stores_and_iterates"
# subject = "http.cookiejar.CookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.CookieJar: set_cookie stores cookies; len() counts them and iterating the jar yields each Cookie (two named cookies -> len 2, names {session, user})"""
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
_jar.set_cookie(_make_cookie("session", "abc123"))
_jar.set_cookie(_make_cookie("user", "alice"))
assert len(_jar) == 2, f"two cookies = {len(_jar)!r}"
_names = {c.name for c in _jar}
assert _names == {"session", "user"}, f"cookie names = {_names!r}"

print("cookiejar_stores_and_iterates OK")
