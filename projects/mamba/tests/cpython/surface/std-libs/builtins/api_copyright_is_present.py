# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "builtins"
# dimension = "surface"
# case = "api_copyright_is_present"
# subject = "builtins.copyright"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""builtins.copyright: api_copyright_is_present (surface)."""
import builtins

assert hasattr(builtins, "copyright")
print("api_copyright_is_present OK")
