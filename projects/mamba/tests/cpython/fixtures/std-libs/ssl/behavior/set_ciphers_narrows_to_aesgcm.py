# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "set_ciphers_narrows_to_aesgcm"
# subject = "ssl.SSLContext.set_ciphers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.set_ciphers: set_ciphers('AESGCM') narrows the suite list to GCM ciphers, keeping at least two of the known AES-GCM suites; ALL and DEFAULT aliases also apply without raising"""
import ssl

_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_ctx.set_ciphers("AESGCM")
_names = {c["name"] for c in _ctx.get_ciphers()}
_expected = {
    "AES128-GCM-SHA256", "ECDHE-ECDSA-AES128-GCM-SHA256",
    "ECDHE-RSA-AES128-GCM-SHA256", "DHE-RSA-AES128-GCM-SHA256",
    "AES256-GCM-SHA384", "ECDHE-ECDSA-AES256-GCM-SHA384",
    "ECDHE-RSA-AES256-GCM-SHA384", "DHE-RSA-AES256-GCM-SHA384",
}
assert len(_names & _expected) >= 2, f"AESGCM keeps GCM suites: {_names & _expected}"

# ALL / DEFAULT aliases apply without raising.
_ctx2 = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_ctx2.set_ciphers("ALL")
_ctx2.set_ciphers("DEFAULT")

print("set_ciphers_narrows_to_aesgcm OK")
