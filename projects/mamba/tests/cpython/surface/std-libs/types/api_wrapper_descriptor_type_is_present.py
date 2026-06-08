# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_wrapper_descriptor_type_is_present"
# subject = "types.WrapperDescriptorType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.WrapperDescriptorType: api_wrapper_descriptor_type_is_present (surface)."""
import types

assert hasattr(types, "WrapperDescriptorType")
print("api_wrapper_descriptor_type_is_present OK")
