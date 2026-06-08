# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_verify_x509_partial_chain_is_present"
# subject = "ssl.VERIFY_X509_PARTIAL_CHAIN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.VERIFY_X509_PARTIAL_CHAIN: api_verify_x509_partial_chain_is_present (surface)."""
import ssl

assert hasattr(ssl, "VERIFY_X509_PARTIAL_CHAIN")
print("api_verify_x509_partial_chain_is_present OK")
