# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_verify_crl_check_chain_is_present"
# subject = "ssl.VERIFY_CRL_CHECK_CHAIN"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.VERIFY_CRL_CHECK_CHAIN: api_verify_crl_check_chain_is_present (surface)."""
import ssl

assert hasattr(ssl, "VERIFY_CRL_CHECK_CHAIN")
print("api_verify_crl_check_chain_is_present OK")
