# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "verify_mode_none_raises"
# subject = "ssl.SSLContext.verify_mode"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.verify_mode: verify_mode_none_raises (errors)."""
import ssl

_raised = False
try:
    setattr(ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER), 'verify_mode', None)
except TypeError:
    _raised = True
assert _raised, "verify_mode_none_raises: expected TypeError"
print("verify_mode_none_raises OK")
