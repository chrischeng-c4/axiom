# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "sslcontext_load_cert_chain_attr"
# subject = "ssl.SSLContext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ssl.SSLContext: sslcontext_load_cert_chain_attr (surface)."""
import ssl

assert hasattr(ssl.SSLContext, "load_cert_chain")
print("sslcontext_load_cert_chain_attr OK")
