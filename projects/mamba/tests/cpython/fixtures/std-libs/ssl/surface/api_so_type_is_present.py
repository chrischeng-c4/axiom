# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_so_type_is_present"
# subject = "ssl.SO_TYPE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.SO_TYPE: api_so_type_is_present (surface)."""
import ssl

assert hasattr(ssl, "SO_TYPE")
print("api_so_type_is_present OK")
