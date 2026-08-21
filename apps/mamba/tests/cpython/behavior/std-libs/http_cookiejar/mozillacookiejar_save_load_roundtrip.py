# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "behavior"
# case = "mozillacookiejar_save_load_roundtrip"
# subject = "http.cookiejar.MozillaCookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.MozillaCookieJar: MozillaCookieJar.save then load (ignore_discard/ignore_expires) round-trips a cookie through a tempfile; the saved cookie name reappears after load"""
import http.cookiejar
import os
import tempfile


def _make_cookie(name, value, domain="example.com", path="/", expires=None, secure=False):
    return http.cookiejar.Cookie(
        version=0, name=name, value=value,
        port=None, port_specified=False,
        domain=domain, domain_specified=True, domain_initial_dot=True,
        path=path, path_specified=True,
        secure=secure, expires=expires, discard=True,
        comment=None, comment_url=None, rest={},
    )


_jar = http.cookiejar.MozillaCookieJar()
_jar.set_cookie(_make_cookie("saved", "value", domain=".example.com"))
with tempfile.NamedTemporaryFile(suffix=".txt", delete=False) as _tf:
    _cookiefile = _tf.name
try:
    _jar.save(_cookiefile, ignore_discard=True, ignore_expires=True)
    assert os.path.exists(_cookiefile), "cookie file created"
    _jar2 = http.cookiejar.MozillaCookieJar(_cookiefile)
    _jar2.load(_cookiefile, ignore_discard=True, ignore_expires=True)
    _loaded = {c.name for c in _jar2}
    assert "saved" in _loaded, f"saved cookie loaded: {_loaded!r}"
finally:
    os.unlink(_cookiefile)

print("mozillacookiejar_save_load_roundtrip OK")
