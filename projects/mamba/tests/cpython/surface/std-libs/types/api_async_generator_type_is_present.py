# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_async_generator_type_is_present"
# subject = "types.AsyncGeneratorType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.AsyncGeneratorType: api_async_generator_type_is_present (surface)."""
import types

assert hasattr(types, "AsyncGeneratorType")
print("api_async_generator_type_is_present OK")
