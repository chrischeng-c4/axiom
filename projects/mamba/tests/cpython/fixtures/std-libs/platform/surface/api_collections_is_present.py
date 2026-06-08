# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_collections_is_present"
# subject = "platform.collections"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.collections: api_collections_is_present (surface)."""
import platform

assert hasattr(platform, "collections")
print("api_collections_is_present OK")
