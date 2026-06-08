# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_report_cdiff_is_present"
# subject = "doctest.REPORT_CDIFF"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.REPORT_CDIFF: api_report_cdiff_is_present (surface)."""
import doctest

assert hasattr(doctest, "REPORT_CDIFF")
print("api_report_cdiff_is_present OK")
