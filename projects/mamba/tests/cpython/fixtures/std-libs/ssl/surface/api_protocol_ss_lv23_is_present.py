# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_protocol_ss_lv23_is_present"
# subject = "ssl.PROTOCOL_SSLv23"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.PROTOCOL_SSLv23: api_protocol_ss_lv23_is_present (surface)."""
import ssl

assert hasattr(ssl, "PROTOCOL_SSLv23")
print("api_protocol_ss_lv23_is_present OK")
