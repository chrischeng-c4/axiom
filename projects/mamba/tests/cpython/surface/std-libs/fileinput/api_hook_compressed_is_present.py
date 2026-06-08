# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fileinput"
# dimension = "surface"
# case = "api_hook_compressed_is_present"
# subject = "fileinput.hook_compressed"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""fileinput.hook_compressed: api_hook_compressed_is_present (surface)."""
import fileinput

assert hasattr(fileinput, "hook_compressed")
print("api_hook_compressed_is_present OK")
