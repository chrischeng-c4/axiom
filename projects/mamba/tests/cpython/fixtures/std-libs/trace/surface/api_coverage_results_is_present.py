# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "trace"
# dimension = "surface"
# case = "api_coverage_results_is_present"
# subject = "trace.CoverageResults"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""trace.CoverageResults: api_coverage_results_is_present (surface)."""
import trace

assert hasattr(trace, "CoverageResults")
print("api_coverage_results_is_present OK")
