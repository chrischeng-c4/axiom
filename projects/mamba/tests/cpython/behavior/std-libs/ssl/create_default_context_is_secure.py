# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "create_default_context_is_secure"
# subject = "ssl.create_default_context"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.create_default_context: create_default_context() yields a secure client context: check_hostname is True and verify_mode is CERT_REQUIRED"""
import ssl

_ctx = ssl.create_default_context()
assert _ctx.check_hostname is True, "check_hostname=True by default"
assert _ctx.verify_mode == ssl.CERT_REQUIRED, "verify_mode=REQUIRED by default"

print("create_default_context_is_secure OK")
