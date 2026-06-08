# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_report_only_first_failure_is_present"
# subject = "doctest.REPORT_ONLY_FIRST_FAILURE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.REPORT_ONLY_FIRST_FAILURE: api_report_only_first_failure_is_present (surface)."""
import doctest

assert hasattr(doctest, "REPORT_ONLY_FIRST_FAILURE")
print("api_report_only_first_failure_is_present OK")
