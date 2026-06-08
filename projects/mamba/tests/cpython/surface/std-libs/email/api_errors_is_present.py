# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email"
# dimension = "surface"
# case = "api_errors_is_present"
# subject = "email.errors"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""email.errors: api_errors_is_present (surface)."""
import email.errors

assert hasattr(email, "errors")
print("api_errors_is_present OK")
