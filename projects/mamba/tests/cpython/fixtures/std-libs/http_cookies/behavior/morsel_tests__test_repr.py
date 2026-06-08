# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "morsel_tests__test_repr"
# subject = "cpython.test_http_cookies.MorselTests.test_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_http_cookies.py::MorselTests::test_repr
"""Auto-ported test: MorselTests::test_repr (CPython 3.12 oracle)."""


import re
from http import cookies


morsel = cookies.Morsel()
assert repr(morsel) == "<Morsel: None=None>"
assert str(morsel) == "Set-Cookie: None=None"
morsel.set("key", "val", "coded_val")
assert repr(morsel) == "<Morsel: key=coded_val>"
assert str(morsel) == "Set-Cookie: key=coded_val"
morsel.update(
    {
        "path": "/",
        "comment": "foo",
        "domain": "example.com",
        "max-age": 0,
        "secure": 0,
        "version": 1,
    }
)
assert (
    repr(morsel)
    == "<Morsel: key=coded_val; Comment=foo; Domain=example.com; "
    "Max-Age=0; Path=/; Version=1>"
)
assert (
    str(morsel)
    == "Set-Cookie: key=coded_val; Comment=foo; Domain=example.com; "
    "Max-Age=0; Path=/; Version=1"
)
morsel["secure"] = True
morsel["httponly"] = 1
assert (
    repr(morsel)
    == "<Morsel: key=coded_val; Comment=foo; Domain=example.com; "
    "HttpOnly; Max-Age=0; Path=/; Secure; Version=1>"
)
assert (
    str(morsel)
    == "Set-Cookie: key=coded_val; Comment=foo; Domain=example.com; "
    "HttpOnly; Max-Age=0; Path=/; Secure; Version=1"
)

morsel = cookies.Morsel()
morsel.set("key", "val", "coded_val")
morsel["expires"] = 0
assert re.search(
    r"<Morsel: key=coded_val; "
    r"expires=\w+, \d+ \w+ \d+ \d+:\d+:\d+ \w+>",
    repr(morsel),
)
assert re.search(
    r"Set-Cookie: key=coded_val; "
    r"expires=\w+, \d+ \w+ \d+ \d+:\d+:\d+ \w+",
    str(morsel),
)

print("MorselTests::test_repr: ok")
