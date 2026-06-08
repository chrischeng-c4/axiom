# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "enabling_check_hostname_forces_required"
# subject = "ssl.SSLContext.check_hostname"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.check_hostname: enabling check_hostname on a CERT_NONE context promotes verify_mode up to CERT_REQUIRED, but leaves an existing CERT_OPTIONAL setting untouched"""
import ssl

# CERT_NONE is promoted to CERT_REQUIRED when check_hostname is enabled.
_ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_ctx.check_hostname = False
_ctx.verify_mode = ssl.CERT_NONE
_ctx.check_hostname = True
assert _ctx.verify_mode == ssl.CERT_REQUIRED, "check_hostname raises verify_mode"

# An existing CERT_OPTIONAL setting is left untouched.
_ctx2 = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
_ctx2.check_hostname = False
_ctx2.verify_mode = ssl.CERT_OPTIONAL
_ctx2.check_hostname = True
assert _ctx2.verify_mode == ssl.CERT_OPTIONAL, "OPTIONAL kept under check_hostname"

print("enabling_check_hostname_forces_required OK")
