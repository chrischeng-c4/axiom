# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_cert_optional_is_present"
# subject = "ssl.CERT_OPTIONAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.CERT_OPTIONAL: api_cert_optional_is_present (surface)."""
import ssl

assert hasattr(ssl, "CERT_OPTIONAL")
print("api_cert_optional_is_present OK")
