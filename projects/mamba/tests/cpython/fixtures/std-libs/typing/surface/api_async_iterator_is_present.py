# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_async_iterator_is_present"
# subject = "typing.AsyncIterator"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.AsyncIterator: api_async_iterator_is_present (surface)."""
import typing

assert hasattr(typing, "AsyncIterator")
print("api_async_iterator_is_present OK")
