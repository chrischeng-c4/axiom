# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_s_iwusr_is_present"
# subject = "stat.S_IWUSR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.S_IWUSR: api_s_iwusr_is_present (surface)."""
import stat

assert hasattr(stat, "S_IWUSR")
print("api_s_iwusr_is_present OK")
