# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copyreg"
# dimension = "surface"
# case = "api_remove_extension_is_present"
# subject = "copyreg.remove_extension"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""copyreg.remove_extension: api_remove_extension_is_present (surface)."""
import copyreg

assert hasattr(copyreg, "remove_extension")
print("api_remove_extension_is_present OK")
