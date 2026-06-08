# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "api_split_result_bytes_is_present"
# subject = "urllib.parse.SplitResultBytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.parse.SplitResultBytes: api_split_result_bytes_is_present (surface)."""
import urllib.parse

assert hasattr(urllib.parse, "SplitResultBytes")
print("api_split_result_bytes_is_present OK")
