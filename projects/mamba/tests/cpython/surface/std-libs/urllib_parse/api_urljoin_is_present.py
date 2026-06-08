# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "api_urljoin_is_present"
# subject = "urllib.parse.urljoin"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""urllib.parse.urljoin: api_urljoin_is_present (surface)."""
import urllib.parse

assert hasattr(urllib.parse, "urljoin")
print("api_urljoin_is_present OK")
