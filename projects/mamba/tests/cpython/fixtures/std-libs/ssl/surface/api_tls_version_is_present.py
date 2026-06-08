# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_tls_version_is_present"
# subject = "ssl.TLSVersion"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.TLSVersion: api_tls_version_is_present (surface)."""
import ssl

assert hasattr(ssl, "TLSVersion")
print("api_tls_version_is_present OK")
