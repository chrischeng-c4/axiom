# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_output_checker_is_present"
# subject = "doctest.OutputChecker"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.OutputChecker: api_output_checker_is_present (surface)."""
import doctest

assert hasattr(doctest, "OutputChecker")
print("api_output_checker_is_present OK")
