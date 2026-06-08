# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copyreg"
# dimension = "surface"
# case = "api_constructor_is_present"
# subject = "copyreg.constructor"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""copyreg.constructor: api_constructor_is_present (surface)."""
import copyreg

assert hasattr(copyreg, "constructor")
print("api_constructor_is_present OK")
