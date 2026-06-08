# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_async_generator_is_present"
# subject = "typing.AsyncGenerator"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.AsyncGenerator: api_async_generator_is_present (surface)."""
import typing

assert hasattr(typing, "AsyncGenerator")
print("api_async_generator_is_present OK")
