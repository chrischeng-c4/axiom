# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_generator_type_is_present"
# subject = "types.GeneratorType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.GeneratorType: api_generator_type_is_present (surface)."""
import types

assert hasattr(types, "GeneratorType")
print("api_generator_type_is_present OK")
