# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copyreg"
# dimension = "surface"
# case = "api_pickle_is_present"
# subject = "copyreg.pickle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""copyreg.pickle: api_pickle_is_present (surface)."""
import copyreg

assert hasattr(copyreg, "pickle")
print("api_pickle_is_present OK")
