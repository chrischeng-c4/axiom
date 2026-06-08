# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "surface"
# case = "api_isfirstline_is_present"
# subject = "fileinput.isfirstline"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""fileinput.isfirstline: api_isfirstline_is_present (surface)."""
import fileinput

assert hasattr(fileinput, "isfirstline")
print("api_isfirstline_is_present OK")
