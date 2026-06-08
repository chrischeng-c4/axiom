# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "surface"
# case = "api_container_is_present"
# subject = "collections.abc.Container"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""collections.abc.Container: api_container_is_present (surface)."""
import collections.abc

assert hasattr(collections.abc, "Container")
print("api_container_is_present OK")
