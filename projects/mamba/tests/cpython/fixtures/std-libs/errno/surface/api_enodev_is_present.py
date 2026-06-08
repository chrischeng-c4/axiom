# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enodev_is_present"
# subject = "errno.ENODEV"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENODEV: api_enodev_is_present (surface)."""
import errno

assert hasattr(errno, "ENODEV")
print("api_enodev_is_present OK")
