# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "bad_protocol_number_raises"
# subject = "ssl.SSLContext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext: bad_protocol_number_raises (errors)."""
import ssl

_raised = False
try:
    ssl.SSLContext(42)
except ValueError:
    _raised = True
assert _raised, "bad_protocol_number_raises: expected ValueError"
print("bad_protocol_number_raises OK")
