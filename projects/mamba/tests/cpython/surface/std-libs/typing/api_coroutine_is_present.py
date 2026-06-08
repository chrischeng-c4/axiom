# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_coroutine_is_present"
# subject = "typing.Coroutine"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Coroutine: api_coroutine_is_present (surface)."""
import typing

assert hasattr(typing, "Coroutine")
print("api_coroutine_is_present OK")
