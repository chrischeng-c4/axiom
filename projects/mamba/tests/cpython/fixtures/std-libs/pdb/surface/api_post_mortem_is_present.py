# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "surface"
# case = "api_post_mortem_is_present"
# subject = "pdb.post_mortem"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pdb.post_mortem: api_post_mortem_is_present (surface)."""
import pdb

assert hasattr(pdb, "post_mortem")
print("api_post_mortem_is_present OK")
