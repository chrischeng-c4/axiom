# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "load_cert_chain_missing_file_raises"
# subject = "ssl.SSLContext.load_cert_chain"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.SSLContext.load_cert_chain: load_cert_chain_missing_file_raises (errors)."""
import ssl

_raised = False
try:
    ssl.create_default_context().load_cert_chain('/no/such/cert.pem')
except FileNotFoundError:
    _raised = True
assert _raised, "load_cert_chain_missing_file_raises: expected FileNotFoundError"
print("load_cert_chain_missing_file_raises OK")
