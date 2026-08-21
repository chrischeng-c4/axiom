# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "surface"
# case = "api_certificate_is_present"
# subject = "_ssl.Certificate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "CPython 3.12 _ssl module"
# status = "filled"
# ///
"""_ssl.Certificate: api_certificate_is_present (surface)."""
import ssl
import _ssl
from _ssl import Certificate

assert ssl._ssl is _ssl
assert hasattr(_ssl, "Certificate")
assert Certificate is _ssl.Certificate
assert Certificate.__module__ == "_ssl"
assert Certificate.__name__ == "Certificate"
assert callable(Certificate)
assert hasattr(Certificate, "public_bytes")
print("api_certificate_is_present OK")
