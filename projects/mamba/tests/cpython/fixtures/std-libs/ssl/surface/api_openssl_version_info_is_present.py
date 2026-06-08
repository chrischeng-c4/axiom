# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_openssl_version_info_is_present"
# subject = "ssl.OPENSSL_VERSION_INFO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.OPENSSL_VERSION_INFO: api_openssl_version_info_is_present (surface)."""
import ssl

assert hasattr(ssl, "OPENSSL_VERSION_INFO")
print("api_openssl_version_info_is_present OK")
