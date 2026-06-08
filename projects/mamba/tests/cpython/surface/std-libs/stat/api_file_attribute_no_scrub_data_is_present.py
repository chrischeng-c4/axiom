# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stat"
# dimension = "surface"
# case = "api_file_attribute_no_scrub_data_is_present"
# subject = "stat.FILE_ATTRIBUTE_NO_SCRUB_DATA"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""stat.FILE_ATTRIBUTE_NO_SCRUB_DATA: api_file_attribute_no_scrub_data_is_present (surface)."""
import stat

assert hasattr(stat, "FILE_ATTRIBUTE_NO_SCRUB_DATA")
print("api_file_attribute_no_scrub_data_is_present OK")
