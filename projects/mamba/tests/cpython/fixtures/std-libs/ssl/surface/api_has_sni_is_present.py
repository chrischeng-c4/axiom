# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_has_sni_is_present"
# subject = "ssl.HAS_SNI"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.HAS_SNI: api_has_sni_is_present (surface)."""
import ssl

assert hasattr(ssl, "HAS_SNI")
print("api_has_sni_is_present OK")
