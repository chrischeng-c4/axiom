# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "api_regex_flag_is_present"
# subject = "re.RegexFlag"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""re.RegexFlag: api_regex_flag_is_present (surface)."""
import re

assert hasattr(re, "RegexFlag")
print("api_regex_flag_is_present OK")
