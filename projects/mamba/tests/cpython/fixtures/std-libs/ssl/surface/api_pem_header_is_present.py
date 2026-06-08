# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_pem_header_is_present"
# subject = "ssl.PEM_HEADER"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.PEM_HEADER: api_pem_header_is_present (surface)."""
import ssl

assert hasattr(ssl, "PEM_HEADER")
print("api_pem_header_is_present OK")
