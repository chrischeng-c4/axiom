# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "surface"
# case = "api_hook_encoded_is_present"
# subject = "fileinput.hook_encoded"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""fileinput.hook_encoded: api_hook_encoded_is_present (surface)."""
import fileinput

assert hasattr(fileinput, "hook_encoded")
print("api_hook_encoded_is_present OK")
