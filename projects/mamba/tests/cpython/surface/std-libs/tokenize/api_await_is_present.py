# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tokenize"
# dimension = "surface"
# case = "api_await_is_present"
# subject = "tokenize.AWAIT"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tokenize.AWAIT: api_await_is_present (surface)."""
import tokenize

assert hasattr(tokenize, "AWAIT")
print("api_await_is_present OK")
