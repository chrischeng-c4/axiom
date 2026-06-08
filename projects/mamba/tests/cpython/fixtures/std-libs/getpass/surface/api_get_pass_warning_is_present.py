# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getpass"
# dimension = "surface"
# case = "api_get_pass_warning_is_present"
# subject = "getpass.GetPassWarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""getpass.GetPassWarning: api_get_pass_warning_is_present (surface)."""
import getpass

assert hasattr(getpass, "GetPassWarning")
print("api_get_pass_warning_is_present OK")
