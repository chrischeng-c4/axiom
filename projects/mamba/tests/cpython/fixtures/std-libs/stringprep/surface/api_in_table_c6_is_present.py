# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "surface"
# case = "api_in_table_c6_is_present"
# subject = "stringprep.in_table_c6"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stringprep.in_table_c6: api_in_table_c6_is_present (surface)."""
import stringprep

assert hasattr(stringprep, "in_table_c6")
print("api_in_table_c6_is_present OK")
