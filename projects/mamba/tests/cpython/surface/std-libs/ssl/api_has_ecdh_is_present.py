# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_has_ecdh_is_present"
# subject = "ssl.HAS_ECDH"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.HAS_ECDH: api_has_ecdh_is_present (surface)."""
import ssl

assert hasattr(ssl, "HAS_ECDH")
print("api_has_ecdh_is_present OK")
