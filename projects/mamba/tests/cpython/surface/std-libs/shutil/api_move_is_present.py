# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_move_is_present"
# subject = "shutil.move"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.move: api_move_is_present (surface)."""
import shutil

assert hasattr(shutil, "move")
print("api_move_is_present OK")
