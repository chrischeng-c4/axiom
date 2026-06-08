# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_rand_status_is_present"
# subject = "ssl.RAND_status"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.RAND_status: api_rand_status_is_present (surface)."""
import ssl

assert hasattr(ssl, "RAND_status")
print("api_rand_status_is_present OK")
