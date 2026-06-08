# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "surface"
# case = "api_frame_info_is_present"
# subject = "inspect.FrameInfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""inspect.FrameInfo: api_frame_info_is_present (surface)."""
import inspect

assert hasattr(inspect, "FrameInfo")
print("api_frame_info_is_present OK")
