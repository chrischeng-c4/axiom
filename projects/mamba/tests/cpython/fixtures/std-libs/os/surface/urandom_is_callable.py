# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "urandom_is_callable"
# subject = "os.urandom"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.urandom: urandom_is_callable (surface)."""
import os

assert callable(os.urandom)
print("urandom_is_callable OK")
