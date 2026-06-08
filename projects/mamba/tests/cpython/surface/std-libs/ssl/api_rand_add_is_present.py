# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_rand_add_is_present"
# subject = "ssl.RAND_add"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.RAND_add: api_rand_add_is_present (surface)."""
import ssl

assert hasattr(ssl, "RAND_add")
print("api_rand_add_is_present OK")
