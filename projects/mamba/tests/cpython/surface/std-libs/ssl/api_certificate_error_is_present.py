# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_certificate_error_is_present"
# subject = "ssl.CertificateError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.CertificateError: api_certificate_error_is_present (surface)."""
import ssl

assert hasattr(ssl, "CertificateError")
print("api_certificate_error_is_present OK")
