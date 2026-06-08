# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "value_non_ascii_octal_escaped"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie/Morsel shells do not octal-escape values in output() (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = "Lib/test/test_http_cookies.py"
# status = "filled"
# ///
"""cookies.SimpleCookie: a non-ASCII value (U+00A9) is backslash-octal escaped (\\251) and double-quoted in output()"""
from http import cookies

c = cookies.SimpleCookie()
c["foo"] = "\u00a9"
assert str(c["foo"]) == 'Set-Cookie: foo="\\251"', f"non-ascii = {str(c['foo'])!r}"
c["foo"]["comment"] = "comment \u00a9"
assert str(c["foo"]) == 'Set-Cookie: foo="\\251"; Comment="comment \\251"', \
    f"comment = {str(c['foo'])!r}"
print("value_non_ascii_octal_escaped OK")
