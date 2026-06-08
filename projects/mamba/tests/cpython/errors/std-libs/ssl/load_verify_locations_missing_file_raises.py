# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "load_verify_locations_missing_file_raises"
# subject = "ssl.SSLContext.load_verify_locations"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.load_verify_locations: load_verify_locations_missing_file_raises (errors)."""
import ssl

_raised = False
try:
    ssl.create_default_context().load_verify_locations('/no/such/ca.pem')
except FileNotFoundError:
    _raised = True
assert _raised, "load_verify_locations_missing_file_raises: expected FileNotFoundError"
print("load_verify_locations_missing_file_raises OK")
