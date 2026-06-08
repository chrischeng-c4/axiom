# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_openssl_version_is_present"
# subject = "ssl.OPENSSL_VERSION"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.OPENSSL_VERSION: api_openssl_version_is_present (surface)."""
import ssl

assert hasattr(ssl, "OPENSSL_VERSION")
print("api_openssl_version_is_present OK")
