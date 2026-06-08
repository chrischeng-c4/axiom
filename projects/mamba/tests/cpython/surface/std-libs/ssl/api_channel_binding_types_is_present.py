# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_channel_binding_types_is_present"
# subject = "ssl.CHANNEL_BINDING_TYPES"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.CHANNEL_BINDING_TYPES: api_channel_binding_types_is_present (surface)."""
import ssl

assert hasattr(ssl, "CHANNEL_BINDING_TYPES")
print("api_channel_binding_types_is_present OK")
