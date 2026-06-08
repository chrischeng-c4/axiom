# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "surface"
# case = "api_file_input_is_present"
# subject = "fileinput.FileInput"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""fileinput.FileInput: api_file_input_is_present (surface)."""
import fileinput

assert hasattr(fileinput, "FileInput")
print("api_file_input_is_present OK")
