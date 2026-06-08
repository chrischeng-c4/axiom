# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_s_irgrp_is_present"
# subject = "stat.S_IRGRP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.S_IRGRP: api_s_irgrp_is_present (surface)."""
import stat

assert hasattr(stat, "S_IRGRP")
print("api_s_irgrp_is_present OK")
