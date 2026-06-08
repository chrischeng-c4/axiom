# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "surface"
# case = "api_isstdin_is_present"
# subject = "fileinput.isstdin"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""fileinput.isstdin: api_isstdin_is_present (surface)."""
import fileinput

assert hasattr(fileinput, "isstdin")
print("api_isstdin_is_present OK")
