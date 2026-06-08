# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "real_world"
# case = "session_cookie_set_and_reparse"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie/Morsel shells have no bound output()/load() and drop attribute state (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.SimpleCookie: a server builds a hardened session cookie (value + Path + Domain + Secure + HttpOnly + Max-Age), emits the Set-Cookie header via output(), and a client reparses the value back via load()"""
from http import cookies

# A server builds a hardened session cookie, serializes the Set-Cookie header,
# and a client reparses the value back from a Cookie header.
server = cookies.SimpleCookie()
server["sid"] = "a1b2c3d4"
server["sid"]["path"] = "/"
server["sid"]["domain"] = "app.example.com"
server["sid"]["secure"] = True
server["sid"]["httponly"] = True
server["sid"]["max-age"] = 3600

header = server.output()
assert header.startswith("Set-Cookie: sid=a1b2c3d4"), f"set-cookie = {header!r}"
assert "Path=/" in header, f"path token: {header!r}"
assert "Domain=app.example.com" in header, f"domain token: {header!r}"
assert "Secure" in header, f"secure token: {header!r}"
assert "HttpOnly" in header, f"httponly token: {header!r}"
assert "Max-Age=3600" in header, f"max-age token: {header!r}"

# The client only echoes back name=value pairs (no attributes) in a Cookie header.
client = cookies.SimpleCookie()
client.load("sid=a1b2c3d4")
assert client["sid"].value == "a1b2c3d4", f"reparsed value = {client['sid'].value!r}"
print("session_cookie_set_and_reparse OK")
