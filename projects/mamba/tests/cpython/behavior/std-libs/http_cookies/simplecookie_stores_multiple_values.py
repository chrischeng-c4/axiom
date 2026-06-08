# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_cookies"
# dimension = "behavior"
# case = "simplecookie_stores_multiple_values"
# subject = "cookies.SimpleCookie"
# kind = "semantic"
# xfail = "mamba SimpleCookie shell drops __setitem__ values and len(); no real dict storage (http_cookies_mod.rs carve-out)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""cookies.SimpleCookie: SimpleCookie holds multiple key=value cookies and reports len() and each Morsel's .value"""
from http import cookies

c = cookies.SimpleCookie()
c["user"] = "alice"
c["token"] = "xyz789"
assert len(c) == 2, f"cookie count = {len(c)!r}"
assert c["user"].value == "alice", f"user = {c['user'].value!r}"
assert c["token"].value == "xyz789", f"token = {c['token'].value!r}"
print("simplecookie_stores_multiple_values OK")
