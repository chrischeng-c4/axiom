# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enxio_is_present"
# subject = "errno.ENXIO"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENXIO: api_enxio_is_present (surface)."""
import errno

assert hasattr(errno, "ENXIO")
print("api_enxio_is_present OK")
