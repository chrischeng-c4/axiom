# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_f_test_is_present"
# subject = "os.F_TEST"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.F_TEST: api_f_test_is_present (surface)."""
import os

assert hasattr(os, "F_TEST")
print("api_f_test_is_present OK")
