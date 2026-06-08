# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_ssl_cert_verification_error_is_present"
# subject = "ssl.SSLCertVerificationError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.SSLCertVerificationError: api_ssl_cert_verification_error_is_present (surface)."""
import ssl

assert hasattr(ssl, "SSLCertVerificationError")
print("api_ssl_cert_verification_error_is_present OK")
