# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_get_set_descriptor_type_is_present"
# subject = "types.GetSetDescriptorType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.GetSetDescriptorType: api_get_set_descriptor_type_is_present (surface)."""
import types

assert hasattr(types, "GetSetDescriptorType")
print("api_get_set_descriptor_type_is_present OK")
