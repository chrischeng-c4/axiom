# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "surface"
# case = "translate_is_callable"
# subject = "fnmatch.translate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""fnmatch.translate: translate_is_callable (surface)."""
import fnmatch

assert callable(fnmatch.translate)
print("translate_is_callable OK")
