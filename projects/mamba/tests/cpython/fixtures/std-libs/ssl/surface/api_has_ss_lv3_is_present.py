# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_has_ss_lv3_is_present"
# subject = "ssl.HAS_SSLv3"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.HAS_SSLv3: api_has_ss_lv3_is_present (surface)."""
import ssl

assert hasattr(ssl, "HAS_SSLv3")
print("api_has_ss_lv3_is_present OK")
