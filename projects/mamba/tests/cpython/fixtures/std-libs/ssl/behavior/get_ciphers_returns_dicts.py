# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "get_ciphers_returns_dicts"
# subject = "ssl.SSLContext.get_ciphers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.get_ciphers: get_ciphers() returns a non-empty list of dicts, each carrying at least 'name' and 'description' keys"""
import ssl

_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_all = _ctx.get_ciphers()
assert isinstance(_all, list) and _all, "get_ciphers returns non-empty list"
assert all("name" in c and "description" in c for c in _all), "cipher dict shape"

print("get_ciphers_returns_dicts OK")
