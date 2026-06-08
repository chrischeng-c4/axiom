# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getpass"
# dimension = "surface"
# case = "api_getuser_is_present"
# subject = "getpass.getuser"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""getpass.getuser: api_getuser_is_present (surface)."""
import getpass

assert hasattr(getpass, "getuser")
print("api_getuser_is_present OK")
