# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_protocol_tls_is_present"
# subject = "ssl.PROTOCOL_TLS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.PROTOCOL_TLS: api_protocol_tls_is_present (surface)."""
import ssl

assert hasattr(ssl, "PROTOCOL_TLS")
print("api_protocol_tls_is_present OK")
