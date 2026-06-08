# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_unregister_unpack_format_is_present"
# subject = "shutil.unregister_unpack_format"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.unregister_unpack_format: api_unregister_unpack_format_is_present (surface)."""
import shutil

assert hasattr(shutil, "unregister_unpack_format")
print("api_unregister_unpack_format_is_present OK")
