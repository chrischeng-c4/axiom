# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "marshal"
# dimension = "surface"
# case = "api_version_is_present"
# subject = "marshal.version"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""marshal.version: api_version_is_present (surface)."""
import marshal

assert hasattr(marshal, "version")
print("api_version_is_present OK")
