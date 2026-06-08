# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "surface"
# case = "api_runcall_is_present"
# subject = "pdb.runcall"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pdb.runcall: api_runcall_is_present (surface)."""
import pdb

assert hasattr(pdb, "runcall")
print("api_runcall_is_present OK")
