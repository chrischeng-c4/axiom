# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_verify_x509_strict_is_present"
# subject = "ssl.VERIFY_X509_STRICT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.VERIFY_X509_STRICT: api_verify_x509_strict_is_present (surface)."""
import ssl

assert hasattr(ssl, "VERIFY_X509_STRICT")
print("api_verify_x509_strict_is_present OK")
