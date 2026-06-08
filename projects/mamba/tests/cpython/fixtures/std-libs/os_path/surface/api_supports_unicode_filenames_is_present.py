# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os_path"
# dimension = "surface"
# case = "api_supports_unicode_filenames_is_present"
# subject = "os.path.supports_unicode_filenames"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.path.supports_unicode_filenames: api_supports_unicode_filenames_is_present (surface)."""
import os.path

assert hasattr(os.path, "supports_unicode_filenames")
print("api_supports_unicode_filenames_is_present OK")
