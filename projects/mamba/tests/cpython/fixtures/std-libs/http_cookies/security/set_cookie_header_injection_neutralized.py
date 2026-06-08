# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "security"
# case = "set_cookie_header_injection_neutralized"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound output() and does not octal-escape or reject hostile cookie values/names (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.SimpleCookie: attacker-controlled CR/LF, NUL/TAB control chars, and '; Path=' attribute-delimiter payloads in a cookie value are octal-escaped and double-quoted by output() so no raw CR/LF or fake attribute survives; a control char in the cookie NAME is rejected with CookieError; a benign value round-trips unquoted"""
from http import cookies

from http.cookies import CookieError

# Untrusted value payloads an attacker would supply to split the header
# or smuggle extra cookie attributes.
ATTACKS = [
    "abc\r\nSet-Cookie: evil=1",        # CRLF response splitting
    "v\nLocation: http://evil",         # bare LF header injection
    "v\rX",                             # bare CR
    "abc\x00def",                       # embedded NUL control char
    "a\tb",                             # embedded TAB control char
    "x; Path=/; Domain=evil.com",       # attribute-delimiter injection
]

for payload in ATTACKS:
    c = cookies.SimpleCookie()
    # CPython does not raise on a hostile *value*; it neutralizes it on
    # serialization. Assigning must not crash.
    c["sid"] = payload
    line = c.output()
    # The header-splitting guarantee: no raw CR or LF survives to output().
    assert "\r" not in line, f"raw CR escaped output: {line!r}"
    assert "\n" not in line, f"raw LF escaped output: {line!r}"
    # A neutralized value is wrapped in double quotes (backslash-octal escaped).
    assert line.startswith('Set-Cookie: sid="'), f"value not quoted: {line!r}"
    # The semicolon that would start a fake "; Path=" attribute is escaped
    # to \073 inside the quotes, so it cannot create a real attribute.
    if ";" in payload:
        assert "; Path=" not in line, f"attribute injection survived: {line!r}"
        assert "\\073" in line, f"semicolon not octal-escaped: {line!r}"

# CRLF specifically becomes \015\012 octal, never a real newline.
crlf = cookies.SimpleCookie()
crlf["sid"] = "abc\r\nSet-Cookie: evil=1"
assert crlf.output() == 'Set-Cookie: sid="abc\\015\\012Set-Cookie: evil=1"', \
    f"crlf encoding = {crlf.output()!r}"

# A control character in the cookie NAME is rejected outright with CookieError
# (names are never quotable, so the only safe action is to refuse).
named = cookies.SimpleCookie()
try:
    named["bad\r\nname"] = "v"
    raised = False
except CookieError:
    raised = True
assert raised, "control char in cookie name was not rejected"

# A benign value round-trips: emitted unquoted, parses back to the same value.
ok = cookies.SimpleCookie()
ok["sid"] = "abc123"
assert ok.output() == "Set-Cookie: sid=abc123", f"benign output = {ok.output()!r}"
assert ok["sid"].value == "abc123"
reparse = cookies.SimpleCookie()
reparse.load("sid=abc123")
assert reparse["sid"].value == "abc123", f"reparse = {reparse['sid'].value!r}"
print("set_cookie_header_injection_neutralized OK")
