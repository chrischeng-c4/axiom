# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "pathlike_is_class"
# subject = "os.PathLike"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.PathLike: pathlike_is_class (surface)."""
import os

assert callable(os.PathLike)
print("pathlike_is_class OK")
