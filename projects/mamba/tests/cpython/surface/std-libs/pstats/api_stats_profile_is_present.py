# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pstats"
# dimension = "surface"
# case = "api_stats_profile_is_present"
# subject = "pstats.StatsProfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""pstats.StatsProfile: api_stats_profile_is_present (surface)."""
import pstats

assert hasattr(pstats, "StatsProfile")
print("api_stats_profile_is_present OK")
