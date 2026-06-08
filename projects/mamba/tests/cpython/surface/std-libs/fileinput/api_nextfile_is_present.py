# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "surface"
# case = "api_nextfile_is_present"
# subject = "fileinput.nextfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""fileinput.nextfile: api_nextfile_is_present (surface)."""
import fileinput

assert hasattr(fileinput, "nextfile")
print("api_nextfile_is_present OK")
