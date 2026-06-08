# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile"
# dimension = "surface"
# case = "api_py_zip_file_is_present"
# subject = "zipfile.PyZipFile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""zipfile.PyZipFile: api_py_zip_file_is_present (surface)."""
import zipfile

assert hasattr(zipfile, "PyZipFile")
print("api_py_zip_file_is_present OK")
