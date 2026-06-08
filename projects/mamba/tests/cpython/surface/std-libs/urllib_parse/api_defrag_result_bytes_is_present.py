# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "api_defrag_result_bytes_is_present"
# subject = "urllib.parse.DefragResultBytes"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.parse.DefragResultBytes: api_defrag_result_bytes_is_present (surface)."""
import urllib.parse

assert hasattr(urllib.parse, "DefragResultBytes")
print("api_defrag_result_bytes_is_present OK")
