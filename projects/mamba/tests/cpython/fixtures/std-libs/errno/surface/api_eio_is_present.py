# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eio_is_present"
# subject = "errno.EIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EIO: api_eio_is_present (surface)."""
import errno

assert hasattr(errno, "EIO")
print("api_eio_is_present OK")
