# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_dotall_is_present"
# subject = "re.DOTALL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.DOTALL: api_dotall_is_present (surface)."""
import re

assert hasattr(re, "DOTALL")
print("api_dotall_is_present OK")
