# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_has_tl_sv1_3_is_present"
# subject = "ssl.HAS_TLSv1_3"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.HAS_TLSv1_3: api_has_tl_sv1_3_is_present (surface)."""
import ssl

assert hasattr(ssl, "HAS_TLSv1_3")
print("api_has_tl_sv1_3_is_present OK")
