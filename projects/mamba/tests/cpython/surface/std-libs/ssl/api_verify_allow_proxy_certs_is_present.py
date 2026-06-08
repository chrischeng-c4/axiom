# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_verify_allow_proxy_certs_is_present"
# subject = "ssl.VERIFY_ALLOW_PROXY_CERTS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.VERIFY_ALLOW_PROXY_CERTS: api_verify_allow_proxy_certs_is_present (surface)."""
import ssl

assert hasattr(ssl, "VERIFY_ALLOW_PROXY_CERTS")
print("api_verify_allow_proxy_certs_is_present OK")
