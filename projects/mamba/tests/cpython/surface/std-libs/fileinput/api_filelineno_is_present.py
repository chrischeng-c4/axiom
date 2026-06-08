# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "surface"
# case = "api_filelineno_is_present"
# subject = "fileinput.filelineno"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""fileinput.filelineno: api_filelineno_is_present (surface)."""
import fileinput

assert hasattr(fileinput, "filelineno")
print("api_filelineno_is_present OK")
