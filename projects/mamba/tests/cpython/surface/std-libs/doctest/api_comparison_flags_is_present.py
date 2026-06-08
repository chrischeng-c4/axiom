# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_comparison_flags_is_present"
# subject = "doctest.COMPARISON_FLAGS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.COMPARISON_FLAGS: api_comparison_flags_is_present (surface)."""
import doctest

assert hasattr(doctest, "COMPARISON_FLAGS")
print("api_comparison_flags_is_present OK")
