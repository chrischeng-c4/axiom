# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "surface"
# case = "api_pdb_is_present"
# subject = "pdb.Pdb"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pdb.Pdb: api_pdb_is_present (surface)."""
import pdb

assert hasattr(pdb, "Pdb")
print("api_pdb_is_present OK")
