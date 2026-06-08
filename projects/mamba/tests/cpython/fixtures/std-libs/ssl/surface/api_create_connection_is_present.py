# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_create_connection_is_present"
# subject = "ssl.create_connection"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.create_connection: api_create_connection_is_present (surface)."""
import ssl

assert hasattr(ssl, "create_connection")
print("api_create_connection_is_present OK")
