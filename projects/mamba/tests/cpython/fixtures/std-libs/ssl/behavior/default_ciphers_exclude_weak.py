# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "default_ciphers_exclude_weak"
# subject = "ssl.SSLContext.get_ciphers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.get_ciphers: the default cipher list of a fresh client context excludes known-weak primitives (PSK, SRP, MD5, RC4, 3DES)"""
import ssl

_default = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
for _suite in _default.get_ciphers():
    _name = _suite["name"]
    for _weak in ("PSK", "SRP", "MD5", "RC4", "3DES"):
        assert _weak not in _name, f"weak cipher {_weak} present in {_name}"

print("default_ciphers_exclude_weak OK")
