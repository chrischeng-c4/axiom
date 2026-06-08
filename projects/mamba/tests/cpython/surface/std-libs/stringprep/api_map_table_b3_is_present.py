# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "surface"
# case = "api_map_table_b3_is_present"
# subject = "stringprep.map_table_b3"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stringprep.map_table_b3: api_map_table_b3_is_present (surface)."""
import stringprep

assert hasattr(stringprep, "map_table_b3")
print("api_map_table_b3_is_present OK")
