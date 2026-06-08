# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_protocol_tls_client_is_present"
# subject = "ssl.PROTOCOL_TLS_CLIENT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.PROTOCOL_TLS_CLIENT: api_protocol_tls_client_is_present (surface)."""
import ssl

assert hasattr(ssl, "PROTOCOL_TLS_CLIENT")
print("api_protocol_tls_client_is_present OK")
