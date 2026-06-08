# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "surface"
# case = "api_hasjrel_is_present"
# subject = "dis.hasjrel"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dis.hasjrel: api_hasjrel_is_present (surface)."""
import dis

assert hasattr(dis, "hasjrel")
print("api_hasjrel_is_present OK")
