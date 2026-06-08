# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "api_build_opener_is_present"
# subject = "urllib.request.build_opener"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.request.build_opener: api_build_opener_is_present (surface)."""
import urllib.request

assert hasattr(urllib.request, "build_opener")
print("api_build_opener_is_present OK")
