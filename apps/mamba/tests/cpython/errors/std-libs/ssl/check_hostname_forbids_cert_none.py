# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "check_hostname_forbids_cert_none"
# subject = "ssl.SSLContext.verify_mode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.verify_mode: with check_hostname enabled, assigning verify_mode = CERT_NONE raises ValueError (hostname checking requires a verifying mode)"""
import ssl

_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_ctx.check_hostname = False
_ctx.verify_mode = ssl.CERT_OPTIONAL
_ctx.check_hostname = True
assert _ctx.verify_mode == ssl.CERT_OPTIONAL, "OPTIONAL kept under check_hostname"
try:
    _ctx.verify_mode = ssl.CERT_NONE
    raise AssertionError("CERT_NONE under check_hostname should raise")
except ValueError:
    pass

print("check_hostname_forbids_cert_none OK")
