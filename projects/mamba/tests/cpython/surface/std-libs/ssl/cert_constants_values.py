# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "cert_constants_values"
# subject = "ssl.CERT_REQUIRED"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.CERT_REQUIRED: the verify-mode constants are the documented small ints: CERT_NONE 0, CERT_OPTIONAL 1, CERT_REQUIRED 2"""
import ssl

assert ssl.CERT_NONE == 0, f"CERT_NONE = {ssl.CERT_NONE!r}"
assert ssl.CERT_OPTIONAL == 1, f"CERT_OPTIONAL = {ssl.CERT_OPTIONAL!r}"
assert ssl.CERT_REQUIRED == 2, f"CERT_REQUIRED = {ssl.CERT_REQUIRED!r}"
assert (ssl.CERT_NONE, ssl.CERT_OPTIONAL, ssl.CERT_REQUIRED) == (0, 1, 2), "CERT_* values"

print("cert_constants_values OK")
