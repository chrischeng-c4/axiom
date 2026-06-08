# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_hascompare_is_present"
# subject = "dis.hascompare"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.hascompare: api_hascompare_is_present (surface)."""
import dis

assert hasattr(dis, "hascompare")
print("api_hascompare_is_present OK")
