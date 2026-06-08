# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pdb"
# dimension = "surface"
# case = "api_run_is_present"
# subject = "pdb.run"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pdb.run: api_run_is_present (surface)."""
import pdb

assert hasattr(pdb, "run")
print("api_run_is_present OK")
