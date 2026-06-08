# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "surface"
# case = "api_outside_destination_error_is_present"
# subject = "tarfile.OutsideDestinationError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""tarfile.OutsideDestinationError: api_outside_destination_error_is_present (surface)."""
import tarfile

assert hasattr(tarfile, "OutsideDestinationError")
print("api_outside_destination_error_is_present OK")
