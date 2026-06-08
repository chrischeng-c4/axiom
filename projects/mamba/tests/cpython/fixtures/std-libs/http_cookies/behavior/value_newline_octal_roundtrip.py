# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "value_newline_octal_roundtrip"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell has no bound load()/output() octal round-trip (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: a literal newline in a loaded value round-trips through \\012 octal encoding between load() and output()"""
from http import cookies

c = cookies.SimpleCookie()
c.load('keebler="E=mc2; L=\\"Loves\\"; fudge=\\012;"')
assert c["keebler"].value == 'E=mc2; L="Loves"; fudge=\n;', \
    f"keebler value = {c['keebler'].value!r}"
assert c.output() == 'Set-Cookie: keebler="E=mc2; L=\\"Loves\\"; fudge=\\012;"', \
    f"keebler output = {c.output()!r}"
print("value_newline_octal_roundtrip OK")
