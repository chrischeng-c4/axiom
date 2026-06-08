# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "surface"
# case = "api_lineno_is_present"
# subject = "fileinput.lineno"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""fileinput.lineno: api_lineno_is_present (surface)."""
import fileinput

assert hasattr(fileinput, "lineno")
print("api_lineno_is_present OK")
