# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "protocol_tls_server_attr"
# subject = "ssl"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""ssl: protocol_tls_server_attr (surface)."""
import ssl

assert hasattr(ssl, "PROTOCOL_TLS_SERVER")
print("protocol_tls_server_attr OK")
