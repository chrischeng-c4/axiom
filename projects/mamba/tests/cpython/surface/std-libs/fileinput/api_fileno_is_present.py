# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "surface"
# case = "api_fileno_is_present"
# subject = "fileinput.fileno"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""fileinput.fileno: api_fileno_is_present (surface)."""
import fileinput

assert hasattr(fileinput, "fileno")
print("api_fileno_is_present OK")
