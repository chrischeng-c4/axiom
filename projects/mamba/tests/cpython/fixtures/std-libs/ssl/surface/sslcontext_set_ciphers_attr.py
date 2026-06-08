# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "sslcontext_set_ciphers_attr"
# subject = "ssl.SSLContext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ssl.SSLContext: sslcontext_set_ciphers_attr (surface)."""
import ssl

assert hasattr(ssl.SSLContext, "set_ciphers")
print("sslcontext_set_ciphers_attr OK")
