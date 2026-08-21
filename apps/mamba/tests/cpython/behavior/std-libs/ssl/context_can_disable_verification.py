# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "context_can_disable_verification"
# subject = "ssl.SSLContext.verify_mode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.verify_mode: verification can be disabled on a context: setting check_hostname False then verify_mode CERT_NONE takes effect"""
import ssl

_ctx = ssl.create_default_context()
_ctx.check_hostname = False
_ctx.verify_mode = ssl.CERT_NONE
assert _ctx.verify_mode == ssl.CERT_NONE, "verify_mode settable to NONE"
assert not _ctx.check_hostname, "check_hostname=False"

print("context_can_disable_verification OK")
