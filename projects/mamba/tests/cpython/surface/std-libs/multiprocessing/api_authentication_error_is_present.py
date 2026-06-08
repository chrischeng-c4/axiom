# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_authentication_error_is_present"
# subject = "multiprocessing.AuthenticationError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.AuthenticationError: api_authentication_error_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "AuthenticationError")
print("api_authentication_error_is_present OK")
