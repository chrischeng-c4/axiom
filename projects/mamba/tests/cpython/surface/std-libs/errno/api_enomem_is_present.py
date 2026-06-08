# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enomem_is_present"
# subject = "errno.ENOMEM"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOMEM: api_enomem_is_present (surface)."""
import errno

assert hasattr(errno, "ENOMEM")
print("api_enomem_is_present OK")
