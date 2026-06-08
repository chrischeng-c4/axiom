# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_enomsg_is_present"
# subject = "errno.ENOMSG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.ENOMSG: api_enomsg_is_present (surface)."""
import errno

assert hasattr(errno, "ENOMSG")
print("api_enomsg_is_present OK")
