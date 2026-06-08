# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "post_handshake_auth_toggles"
# subject = "ssl.SSLContext.post_handshake_auth"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.post_handshake_auth: post_handshake_auth defaults to False on both client and server contexts, toggles True/False independently, and survives a verify_mode change"""
import ssl

for _proto in (ssl.PROTOCOL_TLS_SERVER, ssl.PROTOCOL_TLS_CLIENT):
    _pha = ssl.SSLContext(_proto)
    assert _pha.post_handshake_auth is False, "pha default False"
    _pha.post_handshake_auth = True
    assert _pha.post_handshake_auth is True, "pha settable True"
    _pha.verify_mode = ssl.CERT_REQUIRED
    assert _pha.post_handshake_auth is True, "pha survives verify_mode change"
    _pha.post_handshake_auth = False
    assert _pha.post_handshake_auth is False, "pha settable back to False"

print("post_handshake_auth_toggles OK")
