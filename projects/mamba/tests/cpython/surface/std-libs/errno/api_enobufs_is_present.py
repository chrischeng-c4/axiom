# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enobufs_is_present"
# subject = "errno.ENOBUFS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOBUFS: api_enobufs_is_present (surface)."""
import errno

assert hasattr(errno, "ENOBUFS")
print("api_enobufs_is_present OK")
