# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "surface"
# case = "api_system_random_is_present"
# subject = "secrets.SystemRandom"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""secrets.SystemRandom: api_system_random_is_present (surface)."""
import secrets

assert hasattr(secrets, "SystemRandom")
print("api_system_random_is_present OK")
