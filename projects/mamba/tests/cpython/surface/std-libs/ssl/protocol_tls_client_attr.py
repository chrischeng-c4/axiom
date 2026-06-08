# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "protocol_tls_client_attr"
# subject = "ssl"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ssl: protocol_tls_client_attr (surface)."""
import ssl

assert hasattr(ssl, "PROTOCOL_TLS_CLIENT")
print("protocol_tls_client_attr OK")
