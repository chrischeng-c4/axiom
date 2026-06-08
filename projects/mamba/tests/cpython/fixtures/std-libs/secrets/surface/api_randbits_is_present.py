# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "surface"
# case = "api_randbits_is_present"
# subject = "secrets.randbits"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""secrets.randbits: api_randbits_is_present (surface)."""
import secrets

assert hasattr(secrets, "randbits")
print("api_randbits_is_present OK")
