# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "clear_frames_is_callable"
# subject = "traceback.clear_frames"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.clear_frames: clear_frames_is_callable (surface)."""
import traceback

assert callable(traceback.clear_frames)
print("clear_frames_is_callable OK")
