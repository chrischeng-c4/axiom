# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "surface"
# case = "api_help_is_present"
# subject = "pdb.help"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pdb.help: api_help_is_present (surface)."""
import pdb

assert hasattr(pdb, "help")
print("api_help_is_present OK")
