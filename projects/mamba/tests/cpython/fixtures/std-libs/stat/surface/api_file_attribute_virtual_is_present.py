# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_file_attribute_virtual_is_present"
# subject = "stat.FILE_ATTRIBUTE_VIRTUAL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.FILE_ATTRIBUTE_VIRTUAL: api_file_attribute_virtual_is_present (surface)."""
import stat

assert hasattr(stat, "FILE_ATTRIBUTE_VIRTUAL")
print("api_file_attribute_virtual_is_present OK")
