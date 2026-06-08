# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "surface"
# case = "api_python_branch_is_present"
# subject = "platform.python_branch"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""platform.python_branch: api_python_branch_is_present (surface)."""
import platform

assert hasattr(platform, "python_branch")
print("api_python_branch_is_present OK")
