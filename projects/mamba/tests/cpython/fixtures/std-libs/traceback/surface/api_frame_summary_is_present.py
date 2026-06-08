# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "api_frame_summary_is_present"
# subject = "traceback.FrameSummary"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""traceback.FrameSummary: api_frame_summary_is_present (surface)."""
import traceback

assert hasattr(traceback, "FrameSummary")
print("api_frame_summary_is_present OK")
