# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextvars"
# dimension = "surface"
# case = "api_token_is_present"
# subject = "contextvars.Token"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""contextvars.Token: api_token_is_present (surface)."""
import contextvars

assert hasattr(contextvars, "Token")
print("api_token_is_present OK")
