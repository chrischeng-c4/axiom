# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_eopnotsupp_is_present"
# subject = "errno.EOPNOTSUPP"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EOPNOTSUPP: api_eopnotsupp_is_present (surface)."""
import errno

assert hasattr(errno, "EOPNOTSUPP")
print("api_eopnotsupp_is_present OK")
