# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_mapping_proxy_type_is_present"
# subject = "types.MappingProxyType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.MappingProxyType: api_mapping_proxy_type_is_present (surface)."""
import types

assert hasattr(types, "MappingProxyType")
print("api_mapping_proxy_type_is_present OK")
