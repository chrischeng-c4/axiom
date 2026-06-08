# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "api_clear_frames_is_present"
# subject = "traceback.clear_frames"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""traceback.clear_frames: api_clear_frames_is_present (surface)."""
import traceback

assert hasattr(traceback, "clear_frames")
print("api_clear_frames_is_present OK")
