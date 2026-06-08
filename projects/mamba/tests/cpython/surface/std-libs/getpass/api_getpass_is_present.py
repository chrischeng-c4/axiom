# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getpass"
# dimension = "surface"
# case = "api_getpass_is_present"
# subject = "getpass.getpass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""getpass.getpass: api_getpass_is_present (surface)."""
import getpass

assert hasattr(getpass, "getpass")
print("api_getpass_is_present OK")
