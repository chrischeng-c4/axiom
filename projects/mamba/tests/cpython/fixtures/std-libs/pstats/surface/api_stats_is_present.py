# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "surface"
# case = "api_stats_is_present"
# subject = "pstats.Stats"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pstats.Stats: api_stats_is_present (surface)."""
import pstats

assert hasattr(pstats, "Stats")
print("api_stats_is_present OK")
