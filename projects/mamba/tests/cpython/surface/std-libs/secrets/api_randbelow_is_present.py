# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "surface"
# case = "api_randbelow_is_present"
# subject = "secrets.randbelow"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""secrets.randbelow: api_randbelow_is_present (surface)."""
import secrets

assert hasattr(secrets, "randbelow")
print("api_randbelow_is_present OK")
