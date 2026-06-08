# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_class_method_descriptor_type_is_present"
# subject = "types.ClassMethodDescriptorType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.ClassMethodDescriptorType: api_class_method_descriptor_type_is_present (surface)."""
import types

assert hasattr(types, "ClassMethodDescriptorType")
print("api_class_method_descriptor_type_is_present OK")
