# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "surface"
# case = "api_pure_posix_path_is_present"
# subject = "pathlib.PurePosixPath"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pathlib.PurePosixPath: api_pure_posix_path_is_present (surface)."""
import pathlib

assert hasattr(pathlib, "PurePosixPath")
print("api_pure_posix_path_is_present OK")
