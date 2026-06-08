# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_ignore_exception_detail_is_present"
# subject = "doctest.IGNORE_EXCEPTION_DETAIL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.IGNORE_EXCEPTION_DETAIL: api_ignore_exception_detail_is_present (surface)."""
import doctest

assert hasattr(doctest, "IGNORE_EXCEPTION_DETAIL")
print("api_ignore_exception_detail_is_present OK")
