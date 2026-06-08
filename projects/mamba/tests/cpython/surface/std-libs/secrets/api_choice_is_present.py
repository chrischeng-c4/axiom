# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "secrets"
# dimension = "surface"
# case = "api_choice_is_present"
# subject = "secrets.choice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""secrets.choice: api_choice_is_present (surface)."""
import secrets

assert hasattr(secrets, "choice")
print("api_choice_is_present OK")
