# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "surface"
# case = "api_namedtuple_is_present"
# subject = "ssl.namedtuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ssl.namedtuple: api_namedtuple_is_present (surface)."""
import ssl

assert hasattr(ssl, "namedtuple")
print("api_namedtuple_is_present OK")
