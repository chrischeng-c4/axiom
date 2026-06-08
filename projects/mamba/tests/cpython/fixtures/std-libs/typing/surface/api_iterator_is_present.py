# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "surface"
# case = "api_iterator_is_present"
# subject = "typing.Iterator"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""typing.Iterator: api_iterator_is_present (surface)."""
import typing

assert hasattr(typing, "Iterator")
print("api_iterator_is_present OK")
