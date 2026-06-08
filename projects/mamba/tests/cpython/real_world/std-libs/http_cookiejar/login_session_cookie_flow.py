# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookiejar"
# dimension = "real_world"
# case = "login_session_cookie_flow"
# subject = "http.cookiejar.CookieJar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_http_cookiejar.py"
# status = "filled"
# ///
"""http.cookiejar.CookieJar: an end-user session flow: a CookieJar collects a login session cookie, persists it via a MozillaCookieJar tempfile, reloads it in a fresh jar, and the session cookie is available for a later request to the same host"""
import http.cookiejar
import os
import tempfile
import urllib.request
from email.message import Message


class _FakeResponse:
    """Minimal urllib response carrying Set-Cookie headers, as a server would."""

    def __init__(self, set_cookie_values):
        self._msg = Message()
        for value in set_cookie_values:
            self._msg["Set-Cookie"] = value

    def info(self):
        return self._msg


# 1. The user logs in: the server returns a session cookie via Set-Cookie.
login_req = urllib.request.Request("http://shop.example.com/login")
login_resp = _FakeResponse(["sessionid=abc123; Path=/; Domain=shop.example.com"])

jar = http.cookiejar.MozillaCookieJar()
jar.extract_cookies(login_resp, login_req)
assert "sessionid" in {c.name for c in jar}, "login cookie collected"

# 2. The session is persisted to disk between runs.
with tempfile.NamedTemporaryFile(suffix=".txt", delete=False) as _tf:
    cookie_path = _tf.name
try:
    jar.save(cookie_path, ignore_discard=True, ignore_expires=True)

    # 3. A fresh process reloads the saved session.
    reloaded = http.cookiejar.MozillaCookieJar(cookie_path)
    reloaded.load(ignore_discard=True, ignore_expires=True)
    saved = {c.name: c.value for c in reloaded}
    assert saved.get("sessionid") == "abc123", f"reloaded session = {saved!r}"

    # 4. A later request to the same host carries the session cookie.
    next_req = urllib.request.Request("http://shop.example.com/cart")
    reloaded.add_cookie_header(next_req)
    sent = next_req.get_header("Cookie")
    assert sent is not None and "sessionid=abc123" in sent, f"Cookie header = {sent!r}"

    # 5. An unrelated host does NOT receive the session cookie.
    other_req = urllib.request.Request("http://evil.example.org/cart")
    reloaded.add_cookie_header(other_req)
    assert other_req.get_header("Cookie") is None, "cookie must not leak cross-host"
finally:
    os.unlink(cookie_path)

print("login_session_cookie_flow OK")
