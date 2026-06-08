# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "cookiejar_clear_removes_all"
# subject = "http.cookiejar.CookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.CookieJar: clear() with no arguments removes every cookie from the jar (len drops to 0)"""
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
_jar.set_cookie(_make_cookie("a", "1"))
_jar.set_cookie(_make_cookie("b", "2"))
_jar.clear()
assert len(_jar) == 0, f"after clear = {len(_jar)!r}"

print("cookiejar_clear_removes_all OK")
