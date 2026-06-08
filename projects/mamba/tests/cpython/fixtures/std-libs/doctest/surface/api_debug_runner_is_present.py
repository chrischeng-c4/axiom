# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_debug_runner_is_present"
# subject = "doctest.DebugRunner"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.DebugRunner: api_debug_runner_is_present (surface)."""
import doctest

assert hasattr(doctest, "DebugRunner")
print("api_debug_runner_is_present OK")
