# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "surface"
# case = "api_frame_type_is_present"
# subject = "types.FrameType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""types.FrameType: api_frame_type_is_present (surface)."""
import types

assert hasattr(types, "FrameType")
print("api_frame_type_is_present OK")
