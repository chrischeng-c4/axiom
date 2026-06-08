# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_cert_none_is_present"
# subject = "ssl.CERT_NONE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.CERT_NONE: api_cert_none_is_present (surface)."""
import ssl

assert hasattr(ssl, "CERT_NONE")
print("api_cert_none_is_present OK")
