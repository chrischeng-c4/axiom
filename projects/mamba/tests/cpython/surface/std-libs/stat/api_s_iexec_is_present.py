# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_s_iexec_is_present"
# subject = "stat.S_IEXEC"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.S_IEXEC: api_s_iexec_is_present (surface)."""
import stat

assert hasattr(stat, "S_IEXEC")
print("api_s_iexec_is_present OK")
