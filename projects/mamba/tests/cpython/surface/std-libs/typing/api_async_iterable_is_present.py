# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_async_iterable_is_present"
# subject = "typing.AsyncIterable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.AsyncIterable: api_async_iterable_is_present (surface)."""
import typing

assert hasattr(typing, "AsyncIterable")
print("api_async_iterable_is_present OK")
