# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookiejar_clear_by_domain"
# subject = "http.cookiejar.CookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.CookieJar: clear(domain) removes only the cookies for that domain, leaving cookies from other domains in place"""
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
_jar.set_cookie(_make_cookie("c1", "v1", domain=".a.com"))
_jar.set_cookie(_make_cookie("c2", "v2", domain=".b.com"))
_jar.set_cookie(_make_cookie("c3", "v3", domain=".a.com"))
_jar.clear(".a.com")
_remaining = {c.name for c in _jar}
assert "c2" in _remaining, "b.com cookie kept"
assert "c1" not in _remaining and "c3" not in _remaining, f"a.com cookies removed: {_remaining!r}"

print("cookiejar_clear_by_domain OK")
