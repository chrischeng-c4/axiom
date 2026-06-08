# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "environ_has_get"
# subject = "os.environ"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.environ: environ_has_get (surface)."""
import os

assert hasattr(os.environ, "get")
print("environ_has_get OK")
