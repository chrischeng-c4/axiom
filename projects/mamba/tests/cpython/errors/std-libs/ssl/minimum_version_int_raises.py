# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "minimum_version_int_raises"
# subject = "ssl.SSLContext.minimum_version"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.minimum_version: minimum_version_int_raises (errors)."""
import ssl

_raised = False
try:
    setattr(ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER), 'minimum_version', 42)
except ValueError:
    _raised = True
assert _raised, "minimum_version_int_raises: expected ValueError"
print("minimum_version_int_raises OK")
