# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_fail_fast_is_present"
# subject = "doctest.FAIL_FAST"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.FAIL_FAST: api_fail_fast_is_present (surface)."""
import doctest

assert hasattr(doctest, "FAIL_FAST")
print("api_fail_fast_is_present OK")
