# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "surface"
# case = "api_ebadmsg_is_present"
# subject = "errno.EBADMSG"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""errno.EBADMSG: api_ebadmsg_is_present (surface)."""
import errno

assert hasattr(errno, "EBADMSG")
print("api_ebadmsg_is_present OK")
