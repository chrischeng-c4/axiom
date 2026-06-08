# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enolck_is_present"
# subject = "errno.ENOLCK"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOLCK: api_enolck_is_present (surface)."""
import errno

assert hasattr(errno, "ENOLCK")
print("api_enolck_is_present OK")
